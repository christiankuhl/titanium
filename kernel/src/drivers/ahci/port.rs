use core::ptr::addr_of_mut;

use crate::memory::{allocate_anywhere, PhysAddr, Translate, VirtAddr};

use super::controller::AHCIController;
use super::structs::*;
use super::{command_list_base, command_table_descriptor, delay, full_memory_barrier, metadata_address};

pub struct AHCIPort {
    pub number: usize,
    parent_number: usize,
    reg_ptr: VirtAddr,
    metadata_addr: PhysAddr,
    wait_for_completion: bool,
    supports_staggered_spin_up: bool,
}

impl<'a> AHCIPort {
    pub fn new(number: usize, parent: &AHCIController, reg_ptr: VirtAddr) -> Self {
        let metadata_addr = allocate_anywhere(metadata_address(parent.number, number), 512, NO_CACHE).translate();
        let supports_staggered_spin_up = (parent.hba().control_regs.cap & (1 << 27)) > 0;
        Self {
            number,
            parent_number: parent.number,
            reg_ptr,
            metadata_addr,
            wait_for_completion: true,
            supports_staggered_spin_up,
        }
    }
    pub fn registers(&self) -> &mut PortRegisters {
        unsafe { &mut *(self.reg_ptr as *mut PortRegisters) }
    }
    fn find_cmd_slot(&self) -> Option<usize> {
        // If not set in SACT and CI, the slot is free
        let mut slots = self.registers().sact | self.registers().ci;
        for i in 0..32 {
            if slots & 1 == 0 {
                return Some(i);
            }
            slots >>= 1;
        }
        None
    }
    pub fn initialize(&mut self) {
        self.power_on();
        self.spin_up();
        self.clear_sata_error_register();
        self.start_fis_receiving();
        self.set_active_state();
        self.registers().is = 0xffffffff;
        self.registers().ie = 0xffffffff;
        full_memory_barrier();
        // This actually enables the port...
        self.start_command_list_processing();
        full_memory_barrier();
    }
    fn power_on(&mut self) {
        if (self.registers().cmd & (1 << 20)) == 0 {
            return;
        }
        self.registers().cmd = self.registers().cmd | (1 << 2)
    }
    fn spin_up(&mut self) {
        if !self.supports_staggered_spin_up {
            return;
        }
        self.registers().cmd = self.registers().cmd | (1 << 1)
    }
    fn clear_sata_error_register(&mut self) {
        self.registers().serr = self.registers().serr;
    }
    fn start_fis_receiving(&mut self) {
        let ptr = addr_of_mut!((*self.registers()).cmd);
        unsafe { ptr.write_volatile(ptr.read() | (1 << 4)) };
    }
    fn set_active_state(&mut self) {
        let ptr = addr_of_mut!((*self.registers()).cmd);
        unsafe { ptr.write_volatile(ptr.read() & 0x0ffffff | (1 << 28)) };
    }
    fn start_command_list_processing(&mut self) {
        let ptr = addr_of_mut!((*self.registers()).cmd);
        unsafe { ptr.write_volatile(ptr.read() | 1) };
    }
    pub fn identify(&mut self) {
        let slot = self.find_cmd_slot().unwrap() as isize;
        self.wait_for_completion = true;
        unsafe {
            let cmd_hdr = command_list_base(self.parent_number, self.number);
            let cmd_hdr = (cmd_hdr as *mut CommandHeader).offset(slot);
            let ptr = addr_of_mut!((*cmd_hdr).prdtl);
            ptr.write_volatile(1);
            let ptr = addr_of_mut!((*cmd_hdr).prdbc);
            ptr.write_volatile(256);
            let cmd_table = command_table_descriptor(self.parent_number, self.number, slot as usize) as *mut CommandTable;
            let ptr = addr_of_mut!((*cmd_table).descriptors[0]);
            ptr.write_volatile(PhysicalRegionDescriptor {
                base_low: self.metadata_addr as u32,
                base_high: 0,
                reserved: 0,
                byte_count: 511,
            });
            let ptr = addr_of_mut!((*cmd_hdr).attributes);
            ptr.write_volatile(5 | 128);
            let fis = addr_of_mut!((*cmd_table).command_fis) as *mut RegisterH2D;
            let ptr = addr_of_mut!((*fis).fis_type);
            ptr.write_volatile(FISType::RegisterH2D);
            let ptr = addr_of_mut!((*fis).pmport);
            ptr.write_volatile(128);
            let ptr = addr_of_mut!((*fis).command);
            ptr.write_volatile(ATA_CMD_IDENTIFY);
            self.spin_until_ready();
            let ptr = addr_of_mut!((*self.registers()).ci);
            ptr.write_volatile(ptr.read() | (1 << slot));
            loop {
                if self.registers().serr != 0 {
                    panic!("Error accessing AHCI device")
                }
                if !self.wait_for_completion {
                    break;
                }
            }
        }
    }
    fn spin_until_ready(&self) {
        let mut spin = 0;
        while (self.registers().tfd & (ATA_SR_BSY | ATA_SR_DRQ) > 0) && spin <= 100 {
            delay(1000);
            spin += 1;
        }
        if spin == 100 {
            panic!("Device is not responding!")
        }
    }
    pub fn finish_initialisation(&mut self) {
        let metadata = unsafe { &*(metadata_address(self.parent_number, self.number) as *const ATAIdentifyBlock) };
        let mut logical_sector_size = 512;
        let mut physical_sector_size = 512;
        if (metadata.physical_sector_size_to_logical_sector_size >> 14) == 1 {
            if (metadata.physical_sector_size_to_logical_sector_size & (1 << 12)) > 0 {
                logical_sector_size = metadata.logical_sector_size;
            }
            if (metadata.physical_sector_size_to_logical_sector_size & (1 << 13)) > 0 {
                physical_sector_size = logical_sector_size << (metadata.physical_sector_size_to_logical_sector_size & 0xf);
            }
        }
        // Check if the device supports LBA48 mode
        let max_addressable_sector = if (metadata.commands_and_feature_sets_supported[1] & (1 << 10)) > 0 {
            metadata.user_addressable_logical_sectors_count
        } else {
            metadata.max_28_bit_addressable_logical_sector as u64
        };
        log!(
            "    ...{} capacity, {} B per log. sector, {} B per phys. sector",
            (max_addressable_sector * logical_sector_size as u64).readable(),
            logical_sector_size,
            physical_sector_size
        );

        // m_connected_device = SATADiskDevice::create(m_parent_handler.hba_controller(), *this, logical_sector_size, max_addressable_sector);
    }
    pub fn handle_interrupt(&mut self) {
        self.wait_for_completion = false;
    }
}

use alloc::string::String;
use alloc::{format, vec};
trait DriveSize {
    fn readable(&self) -> String;
}

impl DriveSize for u64 {
    fn readable(&self) -> String {
        let units = vec!["B", "KB", "MB", "GB", "TB"];
        let mut result = String::new();
        for (ord, unit) in units.iter().enumerate() {
            let value = (*self as f32) / ((1 << (10 * ord)) as f32);
            result = format!("{:.2} {}", value, unit);
            if value <= 1024.0 {
                break;
            }
        }
        result
    }
}
