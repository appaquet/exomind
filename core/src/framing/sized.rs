use super::{
    check_from_size, check_into_size, check_offset_substract, Error, FrameBuilder, FrameReader,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io;

/// Frame that encode the size of the underlying frame so that it can expose the
/// exact data when decoding. The size is prepended and appended to support to
/// support iteration in both directions.
pub struct SizedFrame<I: FrameReader> {
    inner: I,
    inner_size: usize,
}

impl<I: FrameReader> SizedFrame<I> {
    pub fn new(inner: I) -> Result<SizedFrame<I>, Error> {
        let mut inner_data = inner.exposed_data();
        check_from_size(4, inner_data)?;

        let inner_size = inner_data.read_u32::<LittleEndian>()? as usize;
        check_from_size(4 + inner_size, inner_data)?;

        Ok(SizedFrame { inner, inner_size })
    }

    pub fn size(&self) -> usize {
        self.inner_size + 4 + 4
    }
}

impl SizedFrame<Vec<u8>> {
    pub fn new_from_reader<R: std::io::Read>(reader: &mut R) -> Result<SizedFrame<Vec<u8>>, Error> {
        let inner_size = reader.read_u32::<LittleEndian>()? as usize;

        let mut buf = vec![0u8; inner_size + 8];

        (&mut buf[0..4]).write_u32::<LittleEndian>(inner_size as u32)?;
        reader.read_exact(&mut buf[4..8 + inner_size])?;

        Ok(SizedFrame {
            inner: buf,
            inner_size,
        })
    }
}

impl SizedFrame<&[u8]> {
    pub fn new_from_next_offset(
        buffer: &[u8],
        next_offset: usize,
    ) -> Result<SizedFrame<&[u8]>, Error> {
        check_offset_substract(next_offset, 4)?;
        check_from_size(next_offset - 4, buffer)?;

        let inner_size = (&buffer[next_offset - 4..]).read_u32::<LittleEndian>()? as usize;
        let offset_subtract = 4 + inner_size + 4;
        check_offset_substract(next_offset, offset_subtract)?;
        let offset = next_offset - offset_subtract;

        SizedFrame::new(&buffer[offset..])
    }
}

impl<I: FrameReader> FrameReader for SizedFrame<I> {
    type OwnedType = SizedFrame<I::OwnedType>;

    fn exposed_data(&self) -> &[u8] {
        &self.inner.exposed_data()[4..4 + self.inner_size]
    }

    fn whole_data(&self) -> &[u8] {
        &self.inner.whole_data()[0..self.inner_size + 8]
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        SizedFrame {
            inner: self.inner.to_owned_frame(),
            inner_size: self.inner_size,
        }
    }
}

impl<I: FrameReader + Clone> Clone for SizedFrame<I> {
    fn clone(&self) -> Self {
        SizedFrame {
            inner: self.inner.clone(),
            inner_size: self.inner_size,
        }
    }
}

/// Sized frame builder
pub struct SizedFrameBuilder<I: FrameBuilder> {
    inner: I,
}

impl<I: FrameBuilder> SizedFrameBuilder<I> {
    pub fn new(inner: I) -> SizedFrameBuilder<I> {
        SizedFrameBuilder { inner }
    }

    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.inner
    }
}

impl<I: FrameBuilder> FrameBuilder for SizedFrameBuilder<I> {
    type OwnedFrameType = SizedFrame<Vec<u8>>;

    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        if let Some(inner_size) = self.inner.expected_size() {
            writer.write_u32::<LittleEndian>(inner_size as u32)?;
            let written_size = self.inner.write_to(writer)?;
            debug_assert_eq!(written_size, inner_size);
            writer.write_u32::<LittleEndian>(inner_size as u32)?;

            Ok(4 + inner_size + 4)
        } else {
            let mut buffer = Vec::new();
            self.inner.write_to(&mut buffer)?;

            writer.write_u32::<LittleEndian>(buffer.len() as u32)?;
            writer.write_all(&buffer)?;
            writer.write_u32::<LittleEndian>(buffer.len() as u32)?;

            Ok(4 + buffer.len() + 4)
        }
    }

    fn write_into(&self, into: &mut [u8]) -> Result<usize, Error> {
        check_into_size(8, into)?;

        let inner_size = self.inner.write_into(&mut into[4..])?;

        (&mut into[0..4]).write_u32::<LittleEndian>(inner_size as u32)?;
        let total_size = inner_size + 8;
        check_into_size(total_size, into)?;
        (&mut into[4 + inner_size..]).write_u32::<LittleEndian>(inner_size as u32)?;

        Ok(total_size)
    }

    fn expected_size(&self) -> Option<usize> {
        self.inner.expected_size().map(|inner_size| inner_size + 8)
    }

    fn as_owned_frame(&self) -> Self::OwnedFrameType {
        SizedFrame::new(self.as_bytes()).expect("Couldn't read just-created frame")
    }
}

/// Iterate through a series of sized frame in the given bytes slice.
pub struct SizedFrameSliceIterator<'a> {
    buffer: &'a [u8],
    current_offset: usize,
    pub last_error: Option<Error>,
}

impl<'a> SizedFrameSliceIterator<'a> {
    pub fn new(buffer: &'a [u8]) -> SizedFrameSliceIterator<'a> {
        SizedFrameSliceIterator {
            buffer,
            current_offset: 0,
            last_error: None,
        }
    }
}

impl<'a> Iterator for SizedFrameSliceIterator<'a> {
    type Item = IteratedSizedSliceFrame<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.current_offset;
        let slice = &self.buffer[offset..];

        match SizedFrame::new(slice) {
            Ok(frame) => {
                self.current_offset += frame.size();
                Some(IteratedSizedSliceFrame { offset, frame })
            }
            Err(err) => {
                self.last_error = Some(err);
                None
            }
        }
    }
}

pub struct IteratedSizedSliceFrame<'a> {
    pub offset: usize,
    pub frame: SizedFrame<&'a [u8]>,
}

/// Iterate through a series of sized frame from given reader.
pub struct SizedFrameReaderIterator<R: std::io::Read> {
    reader: R,
    current_offset: usize,
    pub last_error: Option<Error>,
}

impl<R: std::io::Read> SizedFrameReaderIterator<R> {
    pub fn new(reader: R) -> SizedFrameReaderIterator<R> {
        SizedFrameReaderIterator {
            reader,
            current_offset: 0,
            last_error: None,
        }
    }
}

impl<R: std::io::Read> Iterator for SizedFrameReaderIterator<R> {
    type Item = IteratedSizedReaderFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.current_offset;

        match SizedFrame::new_from_reader(&mut self.reader) {
            Ok(frame) => {
                self.current_offset += frame.size();
                Some(IteratedSizedReaderFrame { offset, frame })
            }
            Err(err) => {
                self.last_error = Some(err);
                None
            }
        }
    }
}

pub struct IteratedSizedReaderFrame {
    pub offset: usize,
    pub frame: SizedFrame<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framing::{assert_builder_equals, CapnpFrameBuilder, TypedCapnpFrame};
    use crate::protos::generated::data_chain_capnp::block_header;
    use std::io::Cursor;

    #[test]
    fn can_build_and_read_sized_inner() -> anyhow::Result<()> {
        let inner = vec![8u8; 100];
        let builder = SizedFrameBuilder::new(inner.clone());
        assert_builder_equals(&builder)?;

        let buf1 = builder.as_bytes();
        let frame_reader = SizedFrame::new(buf1.clone())?;
        assert_eq!(inner, frame_reader.exposed_data());

        let frame_reader_owned = frame_reader.to_owned_frame();
        assert_eq!(inner, frame_reader_owned.exposed_data());

        let mut buf3 = Vec::new();
        frame_reader.copy_to(&mut buf3)?;
        assert_eq!(buf1, buf3);

        assert_eq!(buf1, frame_reader.whole_data());

        let mut buf4 = vec![0u8; 1000];
        let written_size = frame_reader.copy_into(&mut buf4)?;
        assert_eq!(buf1, &buf4[0..written_size]);

        Ok(())
    }

    #[test]
    fn can_build_and_read_unsized_inner() -> anyhow::Result<()> {
        // capnp builder cannot provide the size of the frame until it's serialized
        let mut capnp_builder = CapnpFrameBuilder::<block_header::Owned>::new();
        let mut msg_builder = capnp_builder.get_builder();
        msg_builder.set_offset(1234);

        let builder = SizedFrameBuilder::new(capnp_builder);
        assert_builder_equals(&builder)?;
        let frame_bytes = builder.as_bytes();

        let frame = TypedCapnpFrame::<_, block_header::Owned>::new(SizedFrame::new(frame_bytes)?)?;
        let msg_reader = frame.get_reader()?;
        assert_eq!(1234, msg_reader.get_offset());

        Ok(())
    }

    #[test]
    fn can_build_to_owned() -> anyhow::Result<()> {
        let builder = SizedFrameBuilder::new(vec![1; 10]);

        let frame = builder.as_owned_frame();
        assert_eq!(vec![1; 10], frame.exposed_data());
        assert_eq!(10, frame.inner_size);

        Ok(())
    }

    #[test]
    fn frame_slice_iterator() -> anyhow::Result<()> {
        let buffer = {
            let buffer = Vec::new();
            let mut buffer_cursor = Cursor::new(buffer);

            let frame1 = SizedFrameBuilder::new(vec![1u8; 10]);
            frame1.write_to(&mut buffer_cursor)?;

            let frame2 = SizedFrameBuilder::new(vec![2u8; 10]);
            frame2.write_to(&mut buffer_cursor)?;

            buffer_cursor.into_inner()
        };

        let iter = SizedFrameSliceIterator::new(&buffer);
        let frames = iter.collect::<Vec<_>>();
        assert_eq!(2, frames.len());
        assert_eq!(vec![1u8; 10], frames[0].frame.exposed_data());
        assert_eq!(vec![2u8; 10], frames[1].frame.exposed_data());

        let empty = Vec::new();
        let iter = SizedFrameSliceIterator::new(&empty);
        assert_eq!(0, iter.count());

        Ok(())
    }

    #[test]
    fn frame_reader_iterator() -> anyhow::Result<()> {
        let buffer = {
            let buffer = Vec::new();
            let mut buffer_cursor = Cursor::new(buffer);

            let frame1 = SizedFrameBuilder::new(vec![1u8; 10]);
            frame1.write_to(&mut buffer_cursor)?;

            let frame2 = SizedFrameBuilder::new(vec![2u8; 10]);
            frame2.write_to(&mut buffer_cursor)?;

            buffer_cursor.into_inner()
        };

        let iter = SizedFrameReaderIterator::new(buffer.as_slice());
        let frames = iter.collect::<Vec<_>>();
        assert_eq!(2, frames.len());
        assert_eq!(vec![1u8; 10], frames[0].frame.exposed_data());
        assert_eq!(vec![2u8; 10], frames[1].frame.exposed_data());

        let empty = Vec::new();
        let iter = SizedFrameReaderIterator::new(empty.as_slice());
        assert_eq!(0, iter.count());

        Ok(())
    }

    #[test]
    fn from_next_offset() -> anyhow::Result<()> {
        let buffer = {
            let buffer = Vec::new();
            let mut buffer_cursor = Cursor::new(buffer);

            let frame1 = SizedFrameBuilder::new(vec![1u8; 10]);
            frame1.write_to(&mut buffer_cursor)?;

            let frame2 = SizedFrameBuilder::new(vec![2u8; 10]);
            frame2.write_to(&mut buffer_cursor)?;

            buffer_cursor.into_inner()
        };

        let frame1 = SizedFrame::new(&buffer[..])?;
        let next_offset = frame1.size();
        let frame1_from_next = SizedFrame::new_from_next_offset(&buffer[..], next_offset)?;
        assert_eq!(1, frame1_from_next.exposed_data()[0]);

        let frame2_from_next = SizedFrame::new_from_next_offset(&buffer[..], buffer.len())?;
        assert_eq!(2, frame2_from_next.exposed_data()[0]);

        Ok(())
    }

    #[test]
    fn invalid_from_next_offset() -> anyhow::Result<()> {
        let frame1 = SizedFrameBuilder::new(vec![1u8; 10]);
        let buffer = frame1.as_bytes();

        let result = SizedFrame::new_from_next_offset(&buffer[..], 1);
        assert!(result.is_err());

        let result = SizedFrame::new_from_next_offset(&buffer[..], buffer.len() + 2);
        assert!(result.is_err());

        let result = SizedFrame::new_from_next_offset(&buffer[..], buffer.len() - 1);
        assert!(result.is_err());

        let result = SizedFrame::new_from_next_offset(&buffer[..], buffer.len());
        assert!(result.is_ok());

        Ok(())
    }
}
