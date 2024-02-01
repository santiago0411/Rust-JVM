use std::fs::File;
use std::io;
use std::io::{Cursor, Read};
use crate::class_file::AttributeInfo;

pub trait ReadFromCursor {
    fn read_u8(&mut self) -> io::Result<u8>;
    fn read_bytes(&mut self, count: usize) -> io::Result<Vec<u8>>;
    fn read_u16(&mut self) -> io::Result<u16>;
    fn read_u32(&mut self) -> io::Result<u32>;
    fn read_f32(&mut self) -> io::Result<f32>;
    fn read_u64(&mut self) -> io::Result<u64>;
    fn read_f64(&mut self) -> io::Result<f64>;
    fn read_string(&mut self, length: usize) -> io::Result<String>;
}

impl ReadFromCursor for Cursor<Vec<u8>> {
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut bytes = [0; 1];
        self.read_exact(&mut bytes)?;
        Ok(bytes[0])
    }

    fn read_bytes(&mut self, count: usize) -> io::Result<Vec<u8>> {
        let mut buf: Vec<u8> = vec![0; count];
        self.read_exact(& mut buf)?;
        Ok(buf)
    }

    fn read_u16(&mut self) -> io::Result<u16> {
        let mut bytes = [0; 2];
        self.read_exact(&mut bytes)?;
        Ok(u16::from_be_bytes(bytes))
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        let mut bytes = [0; 4];
        self.read_exact(&mut bytes)?;
        Ok(u32::from_be_bytes(bytes))
    }

    fn read_f32(&mut self) -> io::Result<f32> {
        let mut bytes = [0; 4];
        self.read_exact(&mut bytes)?;
        Ok(f32::from_be_bytes(bytes))
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        let mut bytes = [0; 8];
        self.read_exact(&mut bytes)?;
        Ok(u64::from_be_bytes(bytes))
    }

    fn read_f64(&mut self) -> io::Result<f64> {
        let mut bytes = [0; 8];
        self.read_exact(&mut bytes)?;
        Ok(f64::from_be_bytes(bytes))
    }

    fn read_string(&mut self, length: usize) -> io::Result<String> {
        let buf = self.read_bytes(length)?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }
}

pub fn read_file_to_buf(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file: File = File::open(file_path)?;
    let mut buffer: Vec<u8>= Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

// This func should probably be in an attributes file
pub fn read_attributes(cursor: &mut Cursor<Vec<u8>>, count: usize) -> io::Result<Vec<AttributeInfo>> {
    let mut attributes: Vec<AttributeInfo> = Vec::with_capacity(count);

    for _ in 0..count {
        let attribute_name_index: u16 = cursor.read_u16()?;
        let length: u32 = cursor.read_u32()?;
        let info: Vec<u8> = cursor.read_bytes(length as usize)?;
        attributes.push(AttributeInfo {
            attribute_name_index,
            info
        });
    }

    return Ok(attributes)
}