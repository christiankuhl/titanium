struct Port {
    portnumber: u16
}

impl Port {
    pub fn new(portnumber: u16) -> Port {
        Port { portnumber }
    }
    pub fn write(&self, data: u8) {
        unsafe {
            asm!("outb {}, {}", in(reg_byte) data, in(reg) self.portnumber);
        }
    }
    pub fn read(&self) -> u8 {
        let mut result;
        unsafe {
            asm!("inb {}, {}", in(reg) self.portnumber, out(reg_byte) result);
        }
        result
    }
}