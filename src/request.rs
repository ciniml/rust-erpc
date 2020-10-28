use num_derive::FromPrimitive;

use crate::codec::{Codec, MessageHeader, CodecError};
use crate::cursor::BufferCursor;
use crate::framed_transport::{FramedTransport, FramedTransportError};
use heapless::{ArrayLength, Vec};
use scopeguard::defer;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum MessageType
{
    InvocationMessage = 0,
    OnewayMessage,
    ReplyMessage,
    NotificationMessage,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RequestError<TransportError>
{
    CodecError(CodecError),
    FramedTransportError(FramedTransportError<TransportError>),
}

impl<FramedTransportError> From<CodecError> for RequestError<FramedTransportError> {
    fn from(err: CodecError) -> Self {
        Self::CodecError(err)
    }
}
impl<TransportError> From<FramedTransportError<TransportError>> for RequestError<TransportError> {
    fn from(err: FramedTransportError<TransportError>) -> Self {
        Self::FramedTransportError(err)
    }
}

pub struct Request<Buffer: AsMut<[u8]>, CodecType: Codec<BufferCursor<Buffer>>> {
    buffer: Option<Buffer>,
    service: u32,
    request: u32,
    sequence: u32,
    is_oneway: bool,
    _marker: core::marker::PhantomData<CodecType>,
}

impl<Buffer: AsMut<[u8]>, CodecType: Codec<BufferCursor<Buffer>>> Request<Buffer, CodecType> {
    pub fn new(buffer: Buffer, service: u32, request: u32, sequence: u32, is_oneway: bool) -> Self {
        Self {
            buffer: Some(buffer),
            service,
            request,
            sequence,
            is_oneway,
            _marker: core::marker::PhantomData{},
        }
    }

    pub fn send_request<Error, Transport: FramedTransport<Error>, Constructor: FnOnce(&mut CodecType) -> Result<(), CodecError>>(&mut self, transport: &mut Transport, constructor: Constructor) -> Result<(), RequestError<Error>> {
        let mut buffer = None;
        core::mem::swap(&mut buffer, &mut self.buffer);
        
        let cursor = BufferCursor::new(buffer.unwrap());
        {
            defer! {
                let mut buffer = Some(cursor.release());
                core::mem::swap(&mut buffer, &mut self.buffer);
            }
            let codec = CodecType::new(&mut cursor);
            
            let message_header = MessageHeader {
                message_type: if self.is_oneway { MessageType::InvocationMessage } else {MessageType::OnewayMessage },
                service: self.service,
                request: self.request,
                sequence: self.sequence,
            };
            codec.start_write_message(&message_header)?;
            constructor(&mut codec)?;
        }
        let position = cursor.get_position();

        transport.send(self.buffer.unwrap().as_mut())?;

        if !self.is_oneway {
            transport.receive(self.buffer.unwrap().as_mut())?;
            
        }
        Ok(())
    }
}