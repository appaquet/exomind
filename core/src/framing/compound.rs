use super::{check_from_size, check_into_size, Error, FrameBuilder, FrameReader};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io;

/// Frame that wraps 2 underlying frame into a single frame (like a tuple)
pub struct CompoundFrame<I: FrameReader> {
    inner: I,
    offset_right: usize,
}

impl<I: FrameReader> CompoundFrame<I> {
    pub fn new(inner: I) -> Result<CompoundFrame<I>, Error> {
        let exposed_data = inner.exposed_data();
        check_from_size(4, exposed_data)?;

        let offset_right =
            (&exposed_data[exposed_data.len() - 4..]).read_u32::<LittleEndian>()? as usize;
        Ok(CompoundFrame {
            inner,
            offset_right,
        })
    }

    pub fn reader_left(&self) -> CompoundSideReader<I> {
        CompoundSideReader {
            frame: self,
            left: true,
        }
    }

    pub fn reader_right(&self) -> CompoundSideReader<I> {
        CompoundSideReader {
            frame: self,
            left: false,
        }
    }
}

pub struct CompoundSideReader<'p, I: FrameReader> {
    frame: &'p CompoundFrame<I>,
    left: bool,
}

impl<'p, I: FrameReader> FrameReader for CompoundSideReader<'p, I> {
    type OwnedType = Vec<u8>;

    fn exposed_data(&self) -> &[u8] {
        let exposed_data = self.frame.inner.exposed_data();
        if self.left {
            &exposed_data[..self.frame.offset_right]
        } else {
            &exposed_data[self.frame.offset_right..exposed_data.len() - 4]
        }
    }

    fn whole_data(&self) -> &[u8] {
        self.frame.inner.whole_data()
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        self.exposed_data().to_vec()
    }
}

/// Compound frame builder
pub struct CompoundFrameBuilder<A: FrameBuilder, B: FrameBuilder> {
    left: A,
    right: B,
}

impl<A: FrameBuilder, B: FrameBuilder> CompoundFrameBuilder<A, B> {
    pub fn new(left: A, right: B) -> CompoundFrameBuilder<A, B> {
        CompoundFrameBuilder { left, right }
    }

    pub fn inner(&mut self) -> &mut A {
        &mut self.left
    }
}

impl<A: FrameBuilder, B: FrameBuilder> FrameBuilder for CompoundFrameBuilder<A, B> {
    type OwnedFrameType = CompoundFrame<Vec<u8>>;

    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let left_size = self.left.write_to(writer)?;
        let right_size = self.right.write_to(writer)?;
        writer.write_u32::<LittleEndian>(left_size as u32)?;

        Ok(left_size + right_size + 4)
    }

    fn write_into(&self, into: &mut [u8]) -> Result<usize, Error> {
        let left_size = self.left.write_into(into)?;
        let right_size = self.right.write_into(&mut into[left_size..])?;

        check_into_size(left_size + right_size + 4, into)?;
        (&mut into[left_size + right_size..]).write_u32::<LittleEndian>(left_size as u32)?;

        Ok(left_size + right_size + 4)
    }

    fn expected_size(&self) -> Option<usize> {
        self.left.expected_size().and_then(|left_size| {
            self.right
                .expected_size()
                .map(|right_size| left_size + right_size + 4)
        })
    }

    fn as_owned_frame(&self) -> Self::OwnedFrameType {
        CompoundFrame::new(self.as_bytes()).expect("Couldn't read just-created frame")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framing::assert_builder_equals;

    #[test]
    fn can_build_and_read() -> anyhow::Result<()> {
        let left = vec![1; 10];
        let right = vec![2; 15];

        let builder = CompoundFrameBuilder::new(left, right);
        assert_builder_equals(&builder)?;

        let mut buffer = Vec::new();
        builder.write_to(&mut buffer)?;

        let frame = CompoundFrame::new(buffer)?;
        assert_eq!(vec![1; 10], frame.reader_left().exposed_data());
        assert_eq!(vec![2; 15], frame.reader_right().exposed_data());

        Ok(())
    }

    #[test]
    fn can_build_to_owned() -> anyhow::Result<()> {
        let left = vec![1; 10];
        let right = vec![2; 15];

        let builder = CompoundFrameBuilder::new(left, right);
        assert_builder_equals(&builder)?;

        let frame = builder.as_owned_frame();
        assert_eq!(vec![1; 10], frame.reader_left().exposed_data());
        assert_eq!(vec![2; 15], frame.reader_right().exposed_data());

        Ok(())
    }
}
