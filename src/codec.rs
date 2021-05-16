use num_traits::FromPrimitive;

use core::iter::Iterator;

use crate::cursor::*;
use crate::request::MessageType;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CodecError {
    InvalidCallback,
    InvalidMessageType,
    Cursor(CursorError),
    Utf8Error(core::str::Utf8Error),
}

impl From<CursorError> for CodecError {
    fn from(error: CursorError) -> Self {
        CodecError::Cursor(error)
    }
}

impl From<core::str::Utf8Error> for CodecError {
    fn from(error: core::str::Utf8Error) -> Self {
        CodecError::Utf8Error(error)
    }
}

#[derive(Copy, Clone)]
pub struct MessageHeader {
    pub message_type: crate::request::MessageType,
    pub service: u32,
    pub request: u32,
    pub sequence: u32,
}

pub trait Codec<CursorType: Cursor> {
    fn detach(self) -> CursorType;

    fn start_write_message(&mut self, message_header: &MessageHeader) -> Result<(), CodecError>;
    fn write_bool(&mut self, value: bool) -> Result<(), CodecError>;
    fn write_i8(&mut self, value: i8) -> Result<(), CodecError>;
    fn write_u8(&mut self, value: u8) -> Result<(), CodecError>;
    fn write_i16(&mut self, value: i16) -> Result<(), CodecError>;
    fn write_u16(&mut self, value: u16) -> Result<(), CodecError>;
    fn write_i32(&mut self, value: i32) -> Result<(), CodecError>;
    fn write_u32(&mut self, value: u32) -> Result<(), CodecError>;
    fn write_i64(&mut self, value: i64) -> Result<(), CodecError>;
    fn write_u64(&mut self, value: u64) -> Result<(), CodecError>;
    fn write_f32(&mut self, value: f32) -> Result<(), CodecError>;
    fn write_f64(&mut self, value: f64) -> Result<(), CodecError>;
    fn write_ptr(&mut self, value: usize) -> Result<(), CodecError>;
    fn write_str(&mut self, value: &str) -> Result<(), CodecError>;
    fn write_binary(&mut self, data: &[u8]) -> Result<(), CodecError>;
    fn start_write_list(&mut self, length: usize) -> Result<(), CodecError>;
    fn start_write_union(&mut self, discriminator: i32) -> Result<(), CodecError>;
    fn write_null_flag(&mut self, is_null: bool) -> Result<(), CodecError>;
    fn write_callback(
        &mut self,
        callback_ids: &[usize],
        callback_id: usize,
    ) -> Result<(), CodecError>;

    fn start_read_message(&mut self) -> Result<MessageHeader, CodecError>;
    fn read_bool(&mut self) -> Result<bool, CodecError>;
    fn read_i8(&mut self) -> Result<i8, CodecError>;
    fn read_u8(&mut self) -> Result<u8, CodecError>;
    fn read_i16(&mut self) -> Result<i16, CodecError>;
    fn read_u16(&mut self) -> Result<u16, CodecError>;
    fn read_i32(&mut self) -> Result<i32, CodecError>;
    fn read_u32(&mut self) -> Result<u32, CodecError>;
    fn read_i64(&mut self) -> Result<i64, CodecError>;
    fn read_u64(&mut self) -> Result<u64, CodecError>;
    fn read_f32(&mut self) -> Result<f32, CodecError>;
    fn read_f64(&mut self) -> Result<f64, CodecError>;
    fn read_ptr(&mut self) -> Result<usize, CodecError>;

    fn read_str<'buffer>(&mut self, buffer: &'buffer mut [u8]) -> Result<&'buffer str, CodecError>;
    fn read_binary<'buffer>(
        &mut self,
        buffer: &'buffer mut [u8],
    ) -> Result<&'buffer [u8], CodecError>;

    fn start_read_list(&mut self) -> Result<usize, CodecError>;
    fn start_read_union(&mut self) -> Result<i32, CodecError>;
    fn read_null_flag(&mut self) -> Result<bool, CodecError>;
    fn read_callback(&mut self, callback_ids: &[usize]) -> Result<usize, CodecError>;
}
pub trait CodecFactory<CursorType: Cursor, CodecType: Codec<CursorType>> {
    fn from_cursor(&mut self, cursor: CursorType) -> CodecType;
}

pub struct BasicCodec<CursorType: Cursor> {
    cursor: CursorType,
}

macro_rules! define_codec_write {
    ($name:ident, $type:ty) => {
        fn $name(&mut self, value: $type) -> Result<(), CodecError> {
            self.cursor.write(&value.to_le_bytes())?;
            Ok(())
        }
    };
}

macro_rules! define_codec_read {
    ($name:ident, $type:ty, $type_name:ident) => {
        fn $name(&mut self) -> Result<$type, CodecError> {
            const SIZE: usize = core::mem::size_of::<$type>();
            let mut buffer = [0u8; SIZE];
            self.cursor.read(&mut buffer)?;
            Ok($type_name::from_le_bytes(buffer))
        }
    };
}

impl<CursorType: Cursor> BasicCodec<CursorType> {
    pub fn new(cursor: CursorType) -> Self {
        Self { cursor }
    }
}
pub struct BasicCodecFactory<CursorType: Cursor> {
    _marker: core::marker::PhantomData<CursorType>,
}
impl<CursorType: Cursor> Default for BasicCodecFactory<CursorType> {
    fn default() -> Self {
        Self::new()
    }
}
impl<CursorType: Cursor> BasicCodecFactory<CursorType> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData {},
        }
    }
}
impl<CursorType: Cursor> CodecFactory<CursorType, BasicCodec<CursorType>>
    for BasicCodecFactory<CursorType>
{
    fn from_cursor(&mut self, cursor: CursorType) -> BasicCodec<CursorType> {
        BasicCodec::new(cursor)
    }
}

impl<CursorType: Cursor> Codec<CursorType> for BasicCodec<CursorType> {
    fn detach(self) -> CursorType {
        self.cursor
    }

    define_codec_write!(write_u8, u8);
    define_codec_write!(write_i8, i8);
    define_codec_write!(write_u16, u16);
    define_codec_write!(write_i16, i16);
    define_codec_write!(write_u32, u32);
    define_codec_write!(write_i32, i32);
    define_codec_write!(write_u64, u64);
    define_codec_write!(write_i64, i64);
    define_codec_write!(write_f32, f32);
    define_codec_write!(write_f64, f64);
    define_codec_write!(write_ptr, usize);

    fn write_str(&mut self, value: &str) -> Result<(), CodecError> {
        self.write_binary(value.as_bytes())?;
        Ok(())
    }
    fn write_binary(&mut self, data: &[u8]) -> Result<(), CodecError> {
        self.write_u32(data.len() as u32)?;
        self.cursor.write(data)?;
        Ok(())
    }
    fn start_write_list(&mut self, length: usize) -> Result<(), CodecError> {
        self.write_u32(length as u32)
    }
    fn start_write_union(&mut self, discriminator: i32) -> Result<(), CodecError> {
        self.write_i32(discriminator)
    }
    fn write_null_flag(&mut self, is_null: bool) -> Result<(), CodecError> {
        self.write_u8(if is_null { 1 } else { 0 })
    }
    fn write_callback(
        &mut self,
        callback_ids: &[usize],
        callback_id: usize,
    ) -> Result<(), CodecError> {
        if callback_ids.is_empty() {
            if callback_ids[0] == callback_id {
                Ok(())
            } else {
                Err(CodecError::InvalidCallback)
            }
        } else {
            let index = callback_ids.iter().position(|id| *id == callback_id);
            match index {
                Some(index) => self.write_u8(index as u8),
                None => Err(CodecError::InvalidCallback),
            }
        }
    }

    fn start_write_message(&mut self, message_header: &MessageHeader) -> Result<(), CodecError> {
        let header = (1u32 << 24)
            | ((message_header.service & 0xff) << 16)
            | ((message_header.request & 0xff) << 8)
            | (message_header.message_type as u32);
        self.write_u32(header)?;
        self.write_u32(message_header.sequence)?;
        Ok(())
    }

    define_codec_read!(read_u8, u8, u8);
    define_codec_read!(read_i8, i8, i8);
    define_codec_read!(read_u16, u16, u16);
    define_codec_read!(read_i16, i16, i16);
    define_codec_read!(read_u32, u32, u32);
    define_codec_read!(read_i32, i32, i32);
    define_codec_read!(read_u64, u64, u64);
    define_codec_read!(read_i64, i64, i64);
    define_codec_read!(read_f32, f32, f32);
    define_codec_read!(read_f64, f64, f64);
    define_codec_read!(read_ptr, usize, usize);

    fn read_str<'buffer>(&mut self, buffer: &'buffer mut [u8]) -> Result<&'buffer str, CodecError> {
        let raw_str = self.read_binary(buffer)?;
        let s = core::str::from_utf8(raw_str)?;
        Ok(s)
    }
    fn read_binary<'buffer>(
        &mut self,
        buffer: &'buffer mut [u8],
    ) -> Result<&'buffer [u8], CodecError> {
        let length = self.read_u32()? as usize;
        self.cursor.read(&mut buffer[0..length])?;
        Ok(&buffer[0..length])
    }

    fn start_read_list(&mut self) -> Result<usize, CodecError> {
        let length = self.read_u32()? as usize;
        Ok(length)
    }
    fn start_read_union(&mut self) -> Result<i32, CodecError> {
        self.read_i32()
    }
    fn read_null_flag(&mut self) -> Result<bool, CodecError> {
        let flag = self.read_u8()?;
        Ok(flag != 0)
    }
    fn read_callback(&mut self, callback_ids: &[usize]) -> Result<usize, CodecError> {
        let index = self.read_u8()? as usize;
        if index >= callback_ids.len() {
            Err(CodecError::InvalidCallback)
        } else {
            Ok(callback_ids[index])
        }
    }
    fn start_read_message(&mut self) -> Result<MessageHeader, CodecError> {
        let header = self.read_u32()?;
        let sequence = self.read_u32()?;
        let message_type = if let Some(message_type) = MessageType::from_u32(header & 0xff) {
            message_type
        } else {
            return Err(CodecError::InvalidMessageType);
        };
        let message_header = MessageHeader {
            service: (header >> 16) & 0xff,
            request: (header >> 8) & 0xff,
            message_type,
            sequence,
        };
        Ok(message_header)
    }

    fn write_bool(&mut self, value: bool) -> Result<(), CodecError> {
        self.write_u8(if value { 1u8 } else { 0u8 })
    }

    fn read_bool(&mut self) -> Result<bool, CodecError> {
        let value = self.read_u8()?;
        Ok(value != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_codec_read_write() -> Result<(), CodecError> {
        let mut buffer = [0u8; 256];
        {
            let cursor = SliceCursor::new(&mut buffer);
            let mut codec = BasicCodec::new(cursor);
            codec.write_i8(0)?;
            codec.write_i8(127)?;
            codec.write_i8(-128)?;
            codec.write_u8(0)?;
            codec.write_u8(1)?;
            codec.write_u8(255)?;

            codec.write_u32(0xdeadbeef)?;

            codec.write_f32(0f32)?;
            codec.write_f32(-1f32)?;
            codec.write_f32(1f32)?;
            codec.write_f32(0.5f32)?;
            codec.write_f32(-0.25f32)?;
            codec.write_f32(core::f32::MIN)?;
            codec.write_f32(-core::f32::MIN)?;
            codec.write_f32(core::f32::MAX)?;
            codec.write_f32(-core::f32::MAX)?;
            codec.write_f32(core::f32::NAN)?;
            codec.write_f32(core::f32::INFINITY)?;
            codec.write_f32(core::f32::NEG_INFINITY)?;

            codec.write_str("HogeFugaPiyo🍣")?;
            codec.write_binary(&[0xdeu8, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0x00])?;

            let message_header = MessageHeader {
                message_type: MessageType::NotificationMessage,
                service: 0xfe,
                request: 0xca,
                sequence: 0xdeadbeefu32,
            };
            codec.start_write_message(&message_header)?;
        }
        {
            let cursor = SliceCursor::new(&mut buffer);
            let mut codec = BasicCodec::new(cursor);
            assert_eq!(codec.read_i8()?, 0i8);
            assert_eq!(codec.read_i8()?, 127i8);
            assert_eq!(codec.read_i8()?, -128i8);
            assert_eq!(codec.read_u8()?, 0u8);
            assert_eq!(codec.read_u8()?, 1u8);
            assert_eq!(codec.read_u8()?, 255u8);

            assert_eq!(codec.read_u32()?, 0xdeadbeef);

            assert_eq!(codec.read_f32()?, 0f32);
            assert_eq!(codec.read_f32()?, -1f32);
            assert_eq!(codec.read_f32()?, 1f32);
            assert_eq!(codec.read_f32()?, 0.5f32);
            assert_eq!(codec.read_f32()?, -0.25f32);
            assert_eq!(codec.read_f32()?, core::f32::MIN);
            assert_eq!(codec.read_f32()?, -core::f32::MIN);
            assert_eq!(codec.read_f32()?, core::f32::MAX);
            assert_eq!(codec.read_f32()?, -core::f32::MAX);
            assert!(codec.read_f32()?.is_nan());
            assert_eq!(codec.read_f32()?, core::f32::INFINITY);
            assert_eq!(codec.read_f32()?, core::f32::NEG_INFINITY);

            let mut binary_buffer = [0u8; 256];
            assert_eq!(codec.read_str(&mut binary_buffer)?, "HogeFugaPiyo🍣");
            assert_eq!(
                codec.read_binary(&mut binary_buffer)?,
                &[0xdeu8, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0x00]
            );

            let message_header = codec.start_read_message()?;
            assert_eq!(
                message_header.message_type,
                MessageType::NotificationMessage
            );
            assert_eq!(message_header.service, 0xfeu32);
            assert_eq!(message_header.request, 0xcau32);
            assert_eq!(message_header.sequence, 0xdeadbeefu32);
        }
        Ok(())
    }
}
