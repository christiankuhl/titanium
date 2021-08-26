#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct MemoryMapHeaderTag {
    tag_type: u32,
    size: u32,
    entry_size: u32,
    entry_version: u32,
}

#[derive(Clone, Copy)]
pub struct MemoryMap {
    _header: usize,
}

#[derive(Clone, Copy)]
pub struct MemoryMapIter {
    _header: usize,
    _current: usize,
}

impl MemoryMapIter {
    #[inline]
    fn header(&self) -> *const MemoryMapHeaderTag {
        self._header as *const MemoryMapHeaderTag
    }
    #[inline]
    fn current(&self) -> *const MemoryRegion {
        self._current as *const MemoryRegion
    }
}

impl MemoryMap {
    pub fn new(addr: usize) -> Self {
        Self { _header: addr }
    }
    pub fn iter(&self) -> MemoryMapIter {
        MemoryMapIter { _header: self._header, _current: unsafe { self.header().offset(1) } as usize }
    }
    #[inline]
    fn header(&self) -> *const MemoryMapHeaderTag {
        self._header as *const MemoryMapHeaderTag
    }
    pub fn empty() -> Self {
        Self { _header: 0 }
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
    pub fn usable(&self) -> bool {
        self.region_type == 1
    }
}

impl Iterator for MemoryMapIter {
    type Item = &'static MemoryRegion;
    fn next(&mut self) -> Option<Self::Item> {
        let header = unsafe { *self.header() };
        if self._current + header.entry_size as usize <= self._header + header.size as usize {
            unsafe {
                self._current = self.current().offset(1) as usize;
                return Some(&*self.current());
            }
        }
        None
    }
}
