pub struct QuasiUART {
    address: u32,
    buffer: [u8; 4],
    len: usize,
}

pub const QUASI_UART_ADDRESS: u32 = 0x0000_0004;

impl QuasiUART {
    #[inline(never)]
    pub fn new(address: u32) -> Self {
        Self {
            address,
            buffer: [0u8; 4],
            len: 0,
        }
    }

    #[inline(never)]
    pub fn write_word(&self, word: u32) {
        let dst = core::ptr::from_exposed_addr_mut::<u32>(self.address as usize);
        unsafe { dst.write_volatile(word) };
    }

    #[inline(never)]
    pub fn write_byte(&mut self, byte: u8) {
        self.buffer[self.len] = byte;
        self.len += 1;
        if self.len == 4 {
            self.len = 0;
            let word = u32::from_le_bytes(self.buffer);
            self.write_word(word);
        }
    }

    pub fn flush(&mut self) {
        for i in self.len..4 {
            self.buffer[i] = 0u8;
        }
        self.len = 0;
        let dst = core::ptr::from_exposed_addr_mut::<u32>(self.address as usize);
        unsafe { dst.write_volatile(u32::from_le_bytes(self.buffer)) };
    }
}

impl core::fmt::Write for QuasiUART {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for c in s.bytes() {
            self.write_byte(c);
        }
        self.write_byte(0u8);
        self.flush();

        Ok(())
    }
}
