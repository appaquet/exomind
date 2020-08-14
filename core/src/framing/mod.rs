use std::io;

pub mod capnp;
pub mod compound;
pub mod error;
pub mod multihash;
pub mod padded;
pub mod sized;

pub use self::capnp::{CapnpFrame, CapnpFrameBuilder, TypedCapnpFrame};
pub use self::multihash::{MultihashFrame, MultihashFrameBuilder};
pub use error::Error;
pub use padded::{PaddedFrame, PaddedFrameBuilder};
pub use sized::{IteratedSizedSliceFrame, SizedFrame, SizedFrameBuilder, SizedFrameSliceIterator};

/// Represents a frame that either wrap another frame or bytes (vec or slice).
///
/// The hierarchy of a FrameReader instances are the opposite of the persisted
/// hierarchy. Ex:
///     Stored: SizedFrame( MultihashFrame( Data ) )
///     Runtime: Data ( MultiHash ( SizedFrame ) )
///
/// The reason for this reversal is that the inner most frame doesn't know how
/// to extract the bytes from the whole frame. Therefor, the wrapping frame
/// "exposes" the bytes to the inner most frames at runtime.
pub trait FrameReader {
    type OwnedType: FrameReader;

    /// Data exposed by this frame to inner frame.
    /// Ex: sized frame will expose the sized data without encoded size numbers
    fn exposed_data(&self) -> &[u8];

    /// Data of the whole frame, not just the exposed data.
    fn whole_data(&self) -> &[u8];

    /// Size of the whole data of the frame.
    #[inline]
    fn whole_data_size(&self) -> usize {
        self.whole_data().len()
    }

    /// Converts the frame to a owned version (without lifetime)
    fn to_owned_frame(&self) -> Self::OwnedType;

    /// Copy the frame into the given writer
    fn copy_to<W: io::Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(self.whole_data())?;
        Ok(())
    }

    /// Copy the frame into the given slice
    fn copy_into(&self, into: &mut [u8]) -> Result<usize, Error> {
        let whole_data = self.whole_data();
        check_into_size(whole_data.len(), into)?;
        into[0..whole_data.len()].copy_from_slice(&whole_data);
        Ok(whole_data.len())
    }
}

impl FrameReader for Vec<u8> {
    type OwnedType = Vec<u8>;

    fn exposed_data(&self) -> &[u8] {
        self.as_slice()
    }

    fn whole_data(&self) -> &[u8] {
        self.as_slice()
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        self.clone()
    }
}

impl FrameReader for &[u8] {
    type OwnedType = Vec<u8>;

    fn exposed_data(&self) -> &[u8] {
        self
    }

    fn whole_data(&self) -> &[u8] {
        self
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        self.to_vec()
    }
}

/// A frame builder can wrap another frame (or just data), and can be wrapped
/// into another frame. The runtime hierarchy is the same as persisted.
///
/// Ex:
///     Stored: SizedFrame(MultihashFrame(Data))
///     Runtime: SizedFrameBuilder(MultihashFrameBuilder(Data))
pub trait FrameBuilder {
    type OwnedFrameType;

    /// Write the frame to the given writer
    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<usize, Error>;

    /// Write the frame into the given bytes slice
    fn write_into(&self, into: &mut [u8]) -> Result<usize, Error>;

    /// Expected size of the frame (in bytes). Optional since some kind of
    /// frames have an unknown size until they are serialized (ex: capnp)
    fn expected_size(&self) -> Option<usize>;

    /// Creates a owned version of this frame, which is usually a FrameReader
    /// implementation
    fn as_owned_frame(&self) -> Self::OwnedFrameType;

    /// Writes the frame into a in-memory buffer and returns it.
    fn as_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.write_to(&mut buffer)
            .expect("Couldn't write frame into in-memory vec");
        buffer
    }
}

/// Implementation of FrameBuilder for a bytes array allow wrapping the content
/// of the array into another frame
impl FrameBuilder for Vec<u8> {
    type OwnedFrameType = Vec<u8>;

    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        writer.write_all(&self)?;
        Ok(self.len())
    }

    fn write_into(&self, into: &mut [u8]) -> Result<usize, Error> {
        check_into_size(self.len(), into)?;
        into[0..self.len()].copy_from_slice(&self);
        Ok(self.len())
    }

    fn expected_size(&self) -> Option<usize> {
        Some(self.len())
    }

    fn as_owned_frame(&self) -> Self::OwnedFrameType {
        self.clone()
    }
}

/// Asserts that the destination array is big enough for the needed size
fn check_into_size(needed: usize, into: &[u8]) -> Result<(), Error> {
    if into.len() < needed {
        Err(Error::DestinationTooSmall(needed, into.len()))
    } else {
        Ok(())
    }
}

/// Asserts that the source array contains at least the needed size
fn check_from_size(needed: usize, from: &[u8]) -> Result<(), Error> {
    if from.len() < needed {
        Err(Error::SourceTooSmall(needed, from.len()))
    } else {
        Ok(())
    }
}

/// Asserts that the given offset can be subtracted from an offset
fn check_offset_substract(offset: usize, sub_offset: usize) -> Result<(), Error> {
    if sub_offset > offset {
        Err(Error::OffsetSubtract(offset, sub_offset))
    } else {
        Ok(())
    }
}

#[cfg(test)]
fn assert_builder_equals<B: FrameBuilder>(frame_builder: &B) -> anyhow::Result<()> {
    let mut buffer1 = Vec::new();
    frame_builder.write_to(&mut buffer1)?;

    assert_ne!(0, buffer1.len());

    let mut buffer2 = vec![0; 500];
    let size = frame_builder.write_into(&mut buffer2)?;
    assert_eq!(&buffer1[..], &buffer2[..size]);

    assert_eq!(frame_builder.as_bytes(), buffer1);

    if let Some(expected_size) = frame_builder.expected_size() {
        assert_eq!(expected_size, buffer1.len());
    }

    Ok(())
}
