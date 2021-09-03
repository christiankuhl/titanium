use core::ptr::addr_of_mut;

use crate::memory::{allocate_identity_mapped, PhysAddr, Translate, VirtAddr};

use super::controller::AHCIController;
use super::structs::*;
use super::{command_list_base, command_table_descriptor, delay, full_memory_barrier, AHCI_BASE};

pub struct AHCIPort {
    pub number: usize,
    reg_ptr: VirtAddr,
    metadata_addr: PhysAddr,
    wait_for_completion: bool,
    supports_staggered_spin_up: bool,
}

impl<'a> AHCIPort {
    pub fn new(number: usize, parent: &AHCIController, reg_ptr: VirtAddr) -> Self {
        let metadata_addr = allocate_identity_mapped(2 * AHCI_BASE + number << 10, 512, NO_CACHE).translate();
        let supports_staggered_spin_up = (parent.hba().control_regs.cap & (1 << 27)) > 0;
        Self { number, reg_ptr, metadata_addr, wait_for_completion: true, supports_staggered_spin_up }
    }
    pub fn registers(&self) -> &mut PortRegisters {
        unsafe { &mut *(self.reg_ptr as *mut PortRegisters) }
    }
    //// Find a free command list slot
    fn find_cmd_slot(&self) -> Option<usize> {
        // If not set in SACT and CI, the slot is free
        let mut slots = self.registers().sact | self.registers().ci;
        for i in 0..32 {
            if slots & 1 == 0 {
                return Some(i);
            }
            slots >>= 1;
        }
        // trace_ahci("Cannot find free command list entry\n");
        None
    }
    pub fn initialize(&mut self) {
        self.power_on();
        self.spin_up();
        self.clear_sata_error_register();
        self.start_fis_receiving();
        self.set_active_state();
        // self.registers().sctl = (self.registers().sctl & 0xfffffff0) | 1;
        // delay(1000);
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
            let cmd_hdr = command_list_base(self.number);
            let cmd_hdr = (cmd_hdr as *mut CommandHeader).offset(slot);
            let ptr = addr_of_mut!((*cmd_hdr).prdtl);
            ptr.write_volatile(1);
            let ptr = addr_of_mut!((*cmd_hdr).prdbc);
            ptr.write_volatile(256);
            let cmd_table = command_table_descriptor(self.number, slot as usize) as *mut CommandTable;
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
    pub fn handle_interrupt(&mut self) {
        self.wait_for_completion = false;
    }
}
