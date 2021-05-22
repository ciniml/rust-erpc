use rust_erpc::framed_transport::UnderlyingTransport;
use std::io::{Error, Read, Write};

pub struct IoTransport<Io: Read + Write> {
    io: Io,
}

impl<Io: Read + Write> IoTransport<Io> {
    pub fn new(io: Io) -> Self {
        Self { io }
    }
}

impl<Io: Read + Write> UnderlyingTransport for IoTransport<Io> {
    type Error = Error;
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.io.read_exact(data)
    }
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.io.write_all(data)
    }
}
