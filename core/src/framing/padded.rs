use super::{check_from_size, check_into_size, Error, FrameBuilder, FrameReader};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io;

/// Frame that pads an underlying frame so that it has a minimum size.
pub struct PaddedFrame<I: FrameReader> {
    inner: I,
    padding_size: usize,
}

impl<I: FrameReader> PaddedFrame<I> {
    pub fn new(inner: I) -> Result<PaddedFrame<I>, Error> {
        let exposed_data = inner.exposed_data();
        check_from_size(4, exposed_data)?;

        let padding_size =
            (&exposed_data[exposed_data.len() - 4..]).read_u32::<LittleEndian>()? as usize;
        Ok(PaddedFrame {
            inner,
            padding_size,
        })
    }
}

impl<I: FrameReader> FrameReader for PaddedFrame<I> {
    type OwnedType = PaddedFrame<I::OwnedType>;

    fn exposed_data(&self) -> &[u8] {
        let exposed_data = self.inner.exposed_data();
        &exposed_data[..exposed_data.len() - 4 - self.padding_size]
    }

    fn whole_data(&self) -> &[u8] {
        self.inner.whole_data()
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        PaddedFrame {
            inner: self.inner.to_owned_frame(),
            padding_size: self.padding_size,
        }
    }
}

impl<I: FrameReader + Clone> Clone for PaddedFrame<I> {
    fn clone(&self) -> Self {
        PaddedFrame {
            inner: self.inner.clone(),
            padding_size: self.padding_size,
        }
    }
}

/// Padded frame builder
pub struct PaddedFrameBuilder<I: FrameBuilder> {
    inner: I,
    minimum_size: usize,
}

impl<I: FrameBuilder> PaddedFrameBuilder<I> {
    pub fn new(inner: I, minimum_size: usize) -> PaddedFrameBuilder<I> {
        PaddedFrameBuilder {
            inner,
            minimum_size,
        }
    }

    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.inner
    }

    pub fn set_minimum_size(&mut self, minimum_size: usize) {
        self.minimum_size = minimum_size;
    }
}

impl<I: FrameBuilder> FrameBuilder for PaddedFrameBuilder<I> {
    type OwnedFrameType = PaddedFrame<Vec<u8>>;

    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let inner_size = self.inner.write_to(writer)?;

        let padding_size = if inner_size < self.minimum_size {
            let required_padding = self.minimum_size - inner_size;
            for _i in 0..required_padding {
                writer.write_u8(0)?;
            }
            required_padding
        } else {
            0
        };

        writer.write_u32::<LittleEndian>(padding_size as u32)?;
        Ok(inner_size + padding_size + 4)
    }

    fn write_into(&self, into: &mut [u8]) -> Result<usize, Error> {
        let inner_size = self.inner.write_into(into)?;

        let padding_size = if inner_size < self.minimum_size {
            let required_padding = self.minimum_size - inner_size;
            check_into_size(inner_size + required_padding, into)?;

            for i in 0..required_padding {
                into[inner_size + i] = 0;
            }
            required_padding
        } else {
            0
        };

        let total_size = inner_size + padding_size + 4;
        check_into_size(padding_size, into)?;
        (&mut into[inner_size + padding_size..]).write_u32::<LittleEndian>(padding_size as u32)?;

        Ok(total_size)
    }

    fn expected_size(&self) -> Option<usize> {
        self.inner.expected_size().map(|inner_size| {
            if inner_size < self.minimum_size {
                self.minimum_size + 4
            } else {
                inner_size + 4
            }
        })
    }

    fn as_owned_frame(&self) -> Self::OwnedFrameType {
        PaddedFrame::new(self.as_bytes()).expect("Couldn't read just-created frame")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framing::assert_builder_equals;

    #[test]
    fn can_build_and_read() -> anyhow::Result<()> {
        let builder = PaddedFrameBuilder::new(vec![1; 10], 0);
        assert_builder_equals(&builder)?;

        let frame = PaddedFrame::new(builder.as_bytes())?;
        assert_eq!(vec![1; 10], frame.exposed_data());

        let builder = PaddedFrameBuilder::new(vec![1; 10], 10);
        assert_builder_equals(&builder)?;
        let frame = PaddedFrame::new(builder.as_bytes())?;
        assert_eq!(0, frame.padding_size);
        assert_eq!(vec![1; 10], frame.exposed_data());

        let builder = PaddedFrameBuilder::new(vec![1; 10], 20);
        assert_builder_equals(&builder)?;
        let frame = PaddedFrame::new(builder.as_bytes())?;
        assert_eq!(vec![1; 10], frame.exposed_data());
        assert_eq!(10, frame.padding_size);
        assert!(frame.whole_data().len() > 20);

        Ok(())
    }

    #[test]
    fn can_build_to_owned() -> anyhow::Result<()> {
        let builder = PaddedFrameBuilder::new(vec![1; 10], 0);
        let frame = builder.as_owned_frame();
        assert_eq!(vec![1; 10], frame.exposed_data());

        Ok(())
    }
}
