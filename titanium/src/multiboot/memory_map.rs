#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct MemoryMapHeaderTag {
    tag_type: u32,
    size: u32,
    entry_size: u32,
    entry_version: u32,
}

pub struct MemoryMap {
    header: *const MemoryMapHeaderTag,
}

pub struct MemoryMapIter {
    header: *const MemoryMapHeaderTag,
    current: *const MemoryRegion,
}

impl MemoryMap {
    pub fn new(addr: usize) -> Self {
        Self { header: addr as *const MemoryMapHeaderTag }
    }
    pub fn iter(&self) -> MemoryMapIter {
        MemoryMapIter { header: self.header, current: unsafe { self.header.offset(1) as *const MemoryRegion } }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryRegion {
    pub base_addr: usize,
    pub length: usize,
    pub region_type: u32,
    _reserved: u32,
}

impl MemoryRegion {
    fn usable(&self) -> bool {
        self.region_type == 1
    }
}

impl Iterator for MemoryMapIter {
    type Item = MemoryRegion; 
    fn next(&mut self) -> Option<MemoryRegion> { 
        let header = unsafe { *self.header };
        if self.current as usize + header.entry_size as usize <= self.header as usize + header.size as usize {
            unsafe { self.current = self.current.offset(1); 
                return Some(*self.current)
            }
        } 
        None
    }
}



