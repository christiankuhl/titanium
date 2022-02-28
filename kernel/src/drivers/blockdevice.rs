use alloc::vec::Vec;

use super::ahci::AHCIController;

pub struct Request {
    request_type: RequestType,
    status: RequestStatus,
    block: usize,
    block_count: usize,
    buffer: Vec<u8>,
}

pub enum RequestType {
    Read,
    Write,
}

pub enum RequestStatus {
    NotStarted,
    Success,
    Failure,
}

pub struct BlockDevice {
    logical_sector_size: usize,
    max_addressable_sector: usize,
}

impl BlockDevice {
    pub fn new(logical_sector_size: u32, max_addressable_sector: u64) -> Self {
        Self { logical_sector_size: logical_sector_size as usize, max_addressable_sector: max_addressable_sector as usize }
    }
}
