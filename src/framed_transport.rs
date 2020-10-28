use crc::crc16::{checksum_x25, make_table};
use core::fmt::Debug;
use crate::codec::*;
use crate::cursor::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FramedTransportError<UnderlyingError> {
    BufferTooShort,
    DataTooLong,
    ChecksumError,
    InvalidHeader,
    UnderlyingError(UnderlyingError),
}

impl<UnderlyingError> From<UnderlyingError> for FramedTransportError<UnderlyingError> {
    fn from(err: UnderlyingError) -> Self {
        Self::UnderlyingError(err)
    }
}

pub trait FramedTransport<UnderlyingError> {
    fn get_max_message_size(&self) -> usize;
    fn send(&mut self, data: &[u8]) -> Result<(), FramedTransportError<UnderlyingError>>;
    fn receive<'buffer>(&mut self, buffer: &'buffer mut [u8]) -> Result<&'buffer [u8], FramedTransportError<UnderlyingError>>;
}

pub trait UnderlyingTransport {
    type Error;
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error>;
}
pub struct BasicFramedTransport<Underlying: UnderlyingTransport> {
    underlying: Underlying
}

impl<Underlying: UnderlyingTransport> BasicFramedTransport<Underlying> {
    pub fn new(underlying: Underlying) -> Self {
        Self {
            underlying,
        }
    }
}

impl<Underlying: UnderlyingTransport> FramedTransport<Underlying::Error> for BasicFramedTransport<Underlying> {
    
    fn get_max_message_size(&self) -> usize {
        65535
    }

    fn send(&mut self, data: &[u8]) -> Result<(), FramedTransportError<Underlying::Error>> {
        let length = data.len();
        if length > self.get_max_message_size() {
            return Err(FramedTransportError::DataTooLong);
        }
        
        let checksum = checksum_x25(data);
        let mut header = [0u8; 4];
        {
            let mut cursor = SliceCursor::new(&mut header);
            let mut codec = BasicCodec::new(&mut cursor);
            codec.write_u16(length as u16).unwrap();
            codec.write_u16(checksum).unwrap();
        }
        self.underlying.write_all(&header)?;
        self.underlying.write_all(data)?;
        Ok(())
    }

    fn receive<'buffer>(&mut self, buffer: &'buffer mut [u8]) -> Result<&'buffer [u8], FramedTransportError<Underlying::Error>> {
        let mut header = [0u8; 4];
        self.underlying.read_exact(&mut header)?;

        let mut cursor = SliceCursor::new(&mut header);
        let mut codec = BasicCodec::new(&mut cursor);
        let length = codec.read_u16().unwrap() as usize;
        let checksum = codec.read_u16().unwrap();

        if length > self.get_max_message_size() {
            return Err(FramedTransportError::InvalidHeader);
        }
        if buffer.len() < length {
            return Err(FramedTransportError::BufferTooShort);
        }
        let buffer_part = &mut buffer[0..length];
        self.underlying.read_exact(buffer_part)?;
        let calculated_checksum = checksum_x25(buffer_part);
        if calculated_checksum != checksum {
            return Err(FramedTransportError::ChecksumError);
        }
        Ok(buffer_part)
    }
}

impl<CursorType: Cursor> UnderlyingTransport for CursorType {
    type Error = CursorError;
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.read(data)?;
        Ok(())
    }
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.write(data)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
mod std_transport {
    use super::{UnderlyingTransport};
    use std::io::{Read, Write, Error};
    pub struct IoTransport<Io: Read+Write> {
        io: Io,
    }

    impl<Io: Read+Write> IoTransport<Io> {
        pub fn new(io: Io) -> Self {
            Self {
                io,
            }
        }
    }

    impl<Io: Read+Write> UnderlyingTransport for IoTransport<Io> {
        type Error = Error;
        fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
            self.io.read_exact(data)
        }
        fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            self.io.write_all(data)
        }
    }
}

#[cfg(feature = "std")]
pub use std_transport::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_result_data(expected: &[u8], actual: &[u8]) {
        assert_eq!(expected.len(), actual.len(), "Result data don't have the same length.");
        assert!(expected.iter().zip(actual.iter()).all(|(e, a)| e == a), "Result data are not equal.");
    }
    #[test]
    fn basic_framed_transport() -> Result<(), FramedTransportError<CursorError>> {
        let mut buffer = [0u8; (4+0) + (4+16) + (4+65535)];
        let data16 = {
            let mut data = [0u8; 16];
            for (index, item) in data.iter_mut().enumerate() {
                *item = (index & 0xff) as u8;
            }
            data
        };
        let data65535 = {
            let mut data = [0u8; 65535];
            for (index, item) in data.iter_mut().enumerate() {
                *item = 255u8 - (index & 0xff) as u8;
            }
            data
        };

        {
            let cursor = SliceCursor::new(&mut buffer);
            let mut transport = BasicFramedTransport::new(cursor);
            
            transport.send(&[])?;
            transport.send(&data16)?;
            transport.send(&data65535)?;

            let large_data = [0u8; 65536];
            assert_eq!(transport.send(&large_data), Err(FramedTransportError::DataTooLong), "Data larger than 65535 bytes did not fail to send.");

            assert_eq!(transport.send(&[]), Err(FramedTransportError::UnderlyingError(CursorError::InsufficientBuffer)), "Send data beyond the underlying buffer size did not fail.");
        }
        {
            let cursor = SliceCursor::new(&mut buffer);
            let mut transport = BasicFramedTransport::new(cursor);
            let mut buffer = [0u8; 65535];
            compare_result_data(&[], transport.receive(&mut buffer)?);
            compare_result_data(&data16, transport.receive(&mut buffer)?);
            compare_result_data(&data65535, transport.receive(&mut buffer)?);
        }
        {
            let cursor = SliceCursor::new(&mut buffer);
            let mut transport = BasicFramedTransport::new(cursor);
            let mut buffer = [0u8; 65534];
            compare_result_data(&[], transport.receive(&mut buffer)?);
            compare_result_data(&data16, transport.receive(&mut buffer)?);
            assert_eq!(transport.receive(&mut buffer), Err(FramedTransportError::BufferTooShort));
        }
        {
            buffer[4+0+4+16+4+1] = !buffer[4+0+4+16+4+1];   // Modify the content of the third frame.
            let cursor = SliceCursor::new(&mut buffer);
            let mut transport = BasicFramedTransport::new(cursor);
            let mut buffer = [0u8; 65535];
            transport.receive(&mut buffer)?;
            transport.receive(&mut buffer)?;
            assert_eq!(transport.receive(&mut buffer), Err(FramedTransportError::ChecksumError));
        }
        Ok(())
        
    }
}
