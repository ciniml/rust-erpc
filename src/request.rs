use num_derive::FromPrimitive;

use crate::codec::{Codec, CodecError, CodecFactory, MessageHeader};
use crate::cursor::BufferCursor;
use crate::framed_transport::{FramedTransport, FramedTransportError};

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum MessageType {
    InvocationMessage = 0,
    OnewayMessage,
    ReplyMessage,
    NotificationMessage,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RequestResponseError<TransportError> {
    InvalidRequest,
    InvalidResponse,
    CodecError(CodecError),
    FramedTransportError(FramedTransportError<TransportError>),
}

impl<FramedTransportError> From<CodecError> for RequestResponseError<FramedTransportError> {
    fn from(err: CodecError) -> Self {
        Self::CodecError(err)
    }
}
impl<TransportError> From<FramedTransportError<TransportError>>
    for RequestResponseError<TransportError>
{
    fn from(err: FramedTransportError<TransportError>) -> Self {
        Self::FramedTransportError(err)
    }
}

pub fn send_message<Error, Transport, Constructor, Buffer, CodecType, CodecFactoryType>(
    transport: &mut Transport,
    buffer: Buffer,
    message_header: &MessageHeader,
    mut codec_factory: CodecFactoryType,
    constructor: Constructor,
) -> Result<(), RequestResponseError<Error>>
where
    Transport: FramedTransport<Error>,
    Constructor: FnOnce(&mut CodecType) -> Result<(), CodecError>,
    Buffer: AsMut<[u8]>,
    CodecType: Codec<BufferCursor<Buffer>>,
    CodecFactoryType: CodecFactory<BufferCursor<Buffer>, CodecType>,
{
    let (position, mut buffer) = {
        let cursor = BufferCursor::new(buffer);
        let mut codec = codec_factory.from_cursor(cursor);

        codec.start_write_message(message_header)?;
        constructor(&mut codec)?;
        let cursor = codec.detach();
        (cursor.get_position(), cursor.release())
    };
    transport.send(&buffer.as_mut()[0..position])?;
    Ok(())
}

pub fn receive_message<'buffer, Error, Transport, CodecType, CodecFactoryType>(
    transport: &mut Transport,
    buffer: &'buffer mut [u8],
    mut codec_factory: CodecFactoryType,
) -> Result<(MessageHeader, CodecType), RequestResponseError<Error>>
where
    Transport: FramedTransport<Error>,
    CodecType: Codec<BufferCursor<&'buffer mut [u8]>>,
    CodecFactoryType: CodecFactory<BufferCursor<&'buffer mut [u8]>, CodecType>,
{
    let length = transport.receive(buffer)?.len();
    let cursor = BufferCursor::new(&mut buffer[0..length]);
    let mut codec = codec_factory.from_cursor(cursor);
    let message_header = codec.start_read_message()?;
    Ok((message_header, codec))
}

pub struct Request {
    service: u32,
    request: u32,
    sequence: u32,
    is_oneway: bool,
}

impl Request {
    pub fn new(service: u32, request: u32, sequence: u32, is_oneway: bool) -> Self {
        Self {
            service,
            request,
            sequence,
            is_oneway,
        }
    }

    pub fn send_request<Error, Transport, Constructor, Buffer, CodecType, CodecFactoryType>(
        &self,
        transport: &mut Transport,
        buffer: Buffer,
        codec_factory: CodecFactoryType,
        constructor: Constructor,
    ) -> Result<(), RequestResponseError<Error>>
    where
        Transport: FramedTransport<Error>,
        Constructor: FnOnce(&mut CodecType) -> Result<(), CodecError>,
        Buffer: AsMut<[u8]>,
        CodecType: Codec<BufferCursor<Buffer>>,
        CodecFactoryType: CodecFactory<BufferCursor<Buffer>, CodecType>,
    {
        let message_header = MessageHeader {
            message_type: if self.is_oneway {
                MessageType::OnewayMessage
            } else {
                MessageType::InvocationMessage
            },
            service: self.service,
            request: self.request,
            sequence: self.sequence,
        };
        send_message(
            transport,
            buffer,
            &message_header,
            codec_factory,
            constructor,
        )
    }

    pub fn receive_request<'buffer, Error, Transport, CodecType, CodecFactoryType>(
        transport: &mut Transport,
        buffer: &'buffer mut [u8],
        codec_factory: CodecFactoryType,
    ) -> Result<(Request, CodecType), RequestResponseError<Error>>
    where
        Transport: FramedTransport<Error>,
        CodecType: Codec<BufferCursor<&'buffer mut [u8]>>,
        CodecFactoryType: CodecFactory<BufferCursor<&'buffer mut [u8]>, CodecType>,
    {
        let (message_header, codec) = receive_message(transport, buffer, codec_factory)?;
        let is_oneway = match message_header.message_type {
            MessageType::InvocationMessage => false,
            MessageType::OnewayMessage => true,
            _ => return Err(RequestResponseError::InvalidRequest),
        };
        let request = Request {
            is_oneway,
            service: message_header.service,
            request: message_header.request,
            sequence: message_header.sequence,
        };
        Ok((request, codec))
    }
}

pub struct Response {
    service: u32,
    request: u32,
    sequence: u32,
    is_notification: bool,
}

impl Response {
    pub fn new(service: u32, request: u32, sequence: u32, is_notification: bool) -> Self {
        Self {
            service,
            request,
            sequence,
            is_notification,
        }
    }
    pub fn send_response<Error, Transport, Constructor, Buffer, CodecType, CodecFactoryType>(
        &self,
        transport: &mut Transport,
        buffer: Buffer,
        codec_factory: CodecFactoryType,
        constructor: Constructor,
    ) -> Result<(), RequestResponseError<Error>>
    where
        Transport: FramedTransport<Error>,
        Constructor: FnOnce(&mut CodecType) -> Result<(), CodecError>,
        Buffer: AsMut<[u8]>,
        CodecType: Codec<BufferCursor<Buffer>>,
        CodecFactoryType: CodecFactory<BufferCursor<Buffer>, CodecType>,
    {
        let message_header = MessageHeader {
            message_type: if self.is_notification {
                MessageType::NotificationMessage
            } else {
                MessageType::ReplyMessage
            },
            service: self.service,
            request: self.request,
            sequence: self.sequence,
        };
        send_message(
            transport,
            buffer,
            &message_header,
            codec_factory,
            constructor,
        )
    }
    pub fn receive_response<'buffer, Error, Transport, CodecType, CodecFactoryType>(
        transport: &mut Transport,
        buffer: &'buffer mut [u8],
        codec_factory: CodecFactoryType,
    ) -> Result<(Response, CodecType), RequestResponseError<Error>>
    where
        Transport: FramedTransport<Error>,
        CodecType: Codec<BufferCursor<&'buffer mut [u8]>>,
        CodecFactoryType: CodecFactory<BufferCursor<&'buffer mut [u8]>, CodecType>,
    {
        let (message_header, codec) = receive_message(transport, buffer, codec_factory)?;
        let is_notification = match message_header.message_type {
            MessageType::NotificationMessage => true,
            MessageType::ReplyMessage => false,
            _ => return Err(RequestResponseError::InvalidResponse),
        };
        let response = Response {
            service: message_header.service,
            request: message_header.request,
            sequence: message_header.sequence,
            is_notification,
        };
        Ok((response, codec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::BasicCodecFactory;
    use crate::cursor::{CursorError, SliceCursor};
    use crate::framed_transport::BasicFramedTransport;

    fn compare_result_data(expected: &[u8], actual: &[u8]) {
        assert_eq!(
            expected.len(),
            actual.len(),
            "Result data don't have the same length."
        );
        assert!(
            expected.iter().zip(actual.iter()).all(|(e, a)| e == a),
            "Result data are not equal."
        );
    }
    #[test]
    fn send_receive() -> Result<(), RequestResponseError<CursorError>> {
        let mut buffer = [0u8; (4 + 0) + (4 + 16) + (4 + 65535)];
        let data16 = {
            let mut data = [0u8; 16];
            for (index, item) in data.iter_mut().enumerate() {
                *item = (index & 0xff) as u8;
            }
            data
        };

        {
            let mut frame_buffer = [0u8; 256];
            let cursor = SliceCursor::new(&mut buffer);
            let mut transport = BasicFramedTransport::new(cursor);

            let request = Request::new(1u32, 2u32, 0u32, false);
            request.send_request(
                &mut transport,
                &mut frame_buffer,
                BasicCodecFactory::new(),
                |codec| codec.write_binary(&data16),
            )?;

            let response = Response::new(4u32, 5u32, 0u32, false);
            response.send_response(
                &mut transport,
                &mut frame_buffer,
                BasicCodecFactory::new(),
                |codec| codec.write_binary(&data16),
            )?;

            let cursor = transport.release();
            assert!(cursor.get_position() > 0);
        }

        {
            let mut frame_buffer = [0u8; 256];
            let cursor = SliceCursor::new(&mut buffer);
            let mut transport = BasicFramedTransport::new(cursor);

            let (request, mut codec) = Request::receive_request(
                &mut transport,
                &mut frame_buffer,
                BasicCodecFactory::new(),
            )?;
            assert_eq!(request.service, 1u32);
            assert_eq!(request.request, 2u32);
            assert_eq!(request.sequence, 0u32);
            assert_eq!(request.is_oneway, false);

            let mut data_buffer = [0u8; 16];
            let data = codec.read_binary(&mut data_buffer)?;
            compare_result_data(&data16, &data);

            let (response, mut codec) = Response::receive_response(
                &mut transport,
                &mut frame_buffer,
                BasicCodecFactory::new(),
            )?;
            assert_eq!(response.service, 4u32);
            assert_eq!(response.request, 5u32);
            assert_eq!(response.sequence, 0u32);
            assert_eq!(response.is_notification, false);

            let mut data_buffer = [0u8; 16];
            let data = codec.read_binary(&mut data_buffer)?;
            compare_result_data(&data16, &data);
        }

        Ok(())
    }
}
