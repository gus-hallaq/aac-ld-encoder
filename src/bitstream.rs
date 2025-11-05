// bitstream.rs - Production-level bitstream writer
use crate::error::{AacLdError, Result};

#[derive(Debug)]
pub struct BitstreamWriter {
    buffer: Vec<u8>,
    bit_pos: usize,
    current_byte: u8,
}

impl BitstreamWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            bit_pos: 0,
            current_byte: 0,
        }
    }

    pub fn write_bits(&mut self, value: u32, num_bits: usize) -> Result<()> {
        if num_bits > 32 {
            return Err(AacLdError::BitstreamError("Cannot write more than 32 bits at once".to_string()));
        }

        let mut remaining_bits = num_bits;
        let mut val = value;

        while remaining_bits > 0 {
            let bits_to_write = remaining_bits.min(8 - self.bit_pos);
            let mask = (1u32 << bits_to_write) - 1;
            let bits = (val >> (remaining_bits - bits_to_write)) & mask;

            self.current_byte |= (bits as u8) << (8 - self.bit_pos - bits_to_write);
            self.bit_pos += bits_to_write;

            if self.bit_pos == 8 {
                self.buffer.push(self.current_byte);
                self.current_byte = 0;
                self.bit_pos = 0;
            }

            remaining_bits -= bits_to_write;
            val &= (1u32 << remaining_bits) - 1;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.bit_pos > 0 {
            self.buffer.push(self.current_byte);
            self.current_byte = 0;
            self.bit_pos = 0;
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<Vec<u8>> {
        self.flush()?;
        Ok(self.buffer)
    }
}