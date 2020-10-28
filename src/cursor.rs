
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CursorError {
    InsufficientBuffer,
    NotEnoughData,
}

pub trait Cursor
{
    fn read<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a [u8], CursorError>;
    fn write(&mut self, data: &[u8]) -> Result<(), CursorError>;
}

pub struct BufferCursor<Buffer: AsMut<[u8]>>
{
    buffer: Buffer,
    position: usize,
}

impl<Buffer: AsMut<[u8]>> BufferCursor<Buffer>
{
    pub fn new(buffer: Buffer) -> Self {
        Self { 
            buffer,
            position: 0,
        }
    }
    pub fn release(self) -> Buffer {
        self.buffer
    }
    pub fn reset(&mut self) {
        self.position = 0;
    }
    pub fn get_position(&self) -> usize { 
        self.position
    }
}

impl<Buffer: AsMut<[u8]>> Cursor for BufferCursor<Buffer>
{
    fn read<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a [u8], CursorError> {
        let mut cursor = SliceCursor::new_with_position(&mut self.buffer, self.position);
        cursor.read(buffer)
    }
    fn write(&mut self, data: &[u8]) -> Result<(), CursorError> {
        let mut cursor = SliceCursor::new_with_position(&mut self.buffer, self.position);
        cursor.write(data)
    }
}

pub struct SliceCursor<'buffer>
{
    buffer: &'buffer mut [u8],
    position: usize,
}

impl<'buffer> SliceCursor<'buffer>
{
    pub fn new(buffer: &'buffer mut [u8]) -> Self {
        Self::new_with_position(buffer, 0)
    }
    pub fn new_with_position(buffer: &'buffer mut [u8], position: usize) -> Self {
        Self {
            buffer,
            position,
        }
    }
    pub fn reset(&mut self) {
        self.position = 0;
    }
    pub fn get_position(&self) -> usize { 
        self.position
    }
}
impl<'buffer> Cursor for SliceCursor<'buffer>
{
    fn read<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a [u8], CursorError> {
        let remaining = self.buffer.len() - self.position;
        let bytes_to_read = core::cmp::min(remaining, buffer.len());
        buffer.copy_from_slice(&self.buffer[self.position..self.position + bytes_to_read]);
        self.position += bytes_to_read;
        Ok(&buffer[0..bytes_to_read])
    }
    fn write(&mut self, data: &[u8]) -> Result<(), CursorError> {
        let bytes_to_write = data.len();
        let remaining = self.buffer.len() - self.position;
        if remaining < bytes_to_write {
            Err(CursorError::InsufficientBuffer)
        }
        else {
            self.buffer[self.position..self.position + bytes_to_write].copy_from_slice(data);
            self.position += bytes_to_write;
            Ok(())
        }
    }
}

