use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut};

use crate::memory::{allocate_anywhere, allocate_identity_mapped, Translate, VirtAddr};

use super::port::AHCIPort;
use super::{
    command_list_base, command_table_descriptor, delay, fis_base, full_memory_barrier, BaseAddressRegister, PCIDevice,
};
use super::{metadata_address, structs::*};

pub struct AHCIController {
    pub number: usize,
    pci: Box<PCIDevice>,
    hba_ptr: VirtAddr,
    ports: Vec<Box<AHCIPort>>,
    taken_ports: u32,
}

impl AHCIController {
    pub fn new(pci: Box<PCIDevice>, number: usize) -> Self {
        let hba_ptr = if let BaseAddressRegister::MemoryMapped(abar) = pci.bar[5] {
            allocate_identity_mapped(abar.base_address(), size_of::<HBA>(), NO_CACHE)
        } else {
            unreachable!()
        };
        Self { number, pci, hba_ptr, ports: Vec::new(), taken_ports: 0 }
    }
    pub fn hba(&self) -> &mut HBA {
        unsafe { &mut *(self.hba_ptr as *mut HBA) }
    }
    pub fn interrupt_vector(&self) -> u8 {
        self.pci.interrupt_vector()
    }
    pub fn initialize(&mut self) {
        // Ensure that HBA knows we are AHCI aware.
        self.hba().control_regs.ghc = 0x80000000;
        // Enable interrupt line
        self.pci.common.command.write(self.pci.common.command.read() & !(1 << 10));
        // Enable bus mastering
        self.pci.common.command.write(self.pci.common.command.read() | (1 << 0) | (1 << 2));
        // Enable global interrupts
        self.hba().control_regs.ghc |= 1 << 1;
        self.enumerate_ports();
    }
    pub fn identify_drives(&mut self) {
        for port in &mut self.ports {
            port.identify();
        }
    }
    fn reset(&mut self) -> bool {
        self.hba().control_regs.ghc = 1;
        full_memory_barrier();
        let mut retry = 0;
        loop {
            if retry > 1000 {
                return false;
            }
            if (self.hba().control_regs.ghc & 1) == 0 {
                break;
            }
            delay(1000);
            retry += 1;
        }
        // The HBA is locked or hung if we waited more than 1 second!
        return true;
    }

    pub fn enumerate_ports(&mut self) {
        let pi = self.hba().control_regs.pi;
        for idx in 0..32 {
            let ssts = self.hba().port_regs[idx].ssts;
            let ipm = (ssts >> 8) & 0xf == HBA_PORT_IPM_ACTIVE;
            let det = ssts & 0xf == HBA_PORT_DET_PRESENT;
            if pi & (1 << idx) > 0 && ipm && det {
                let signature = self.hba().port_regs[idx].sig;
                match signature {
                    SATA_DRIVE => {
                        log!("    SATA drive detected on port {}", idx);
                        let reg_ptr = addr_of!(self.hba().port_regs[idx]) as VirtAddr;
                        let mut port = AHCIPort::new(idx, self, reg_ptr);
                        port.initialize();
                        self.ports.push(Box::new(port));
                        self.taken_ports += 1 << idx;
                    }
                    ATAPI_DRIVE => {
                        log!("    ATAPI drive detected on port {}", idx);
                    }
                    EM_BRIDGE => {
                        log!("    Enclosure management bridge detected on port {}", idx);
                    }
                    PORT_MULTIPLIER => {
                        log!("    Port multiplier detected on port {}", idx);
                    }
                    _ => {
                        log!("    Garbage device signature {:#x} on port {} - ignored", signature, idx);
                    }
                }
            }
        }
        for port in &self.ports {
            self.rebase(port);
        }
    }
    fn stop_port(&self, idx: usize) {
        let ptr = addr_of_mut!(self.hba().port_regs[idx].cmd);
        unsafe {
            ptr.write_volatile(ptr.read_volatile() & !HBA_PXCMD_ST);
            ptr.write_volatile(ptr.read_volatile() & !HBA_PXCMD_FRE);
            loop {
                if (ptr.read_volatile() & HBA_PXCMD_FR) > 0 {
                    continue;
                }
                if (ptr.read_volatile() & HBA_PXCMD_CR) > 0 {
                    continue;
                }
                break;
            }
        }
    }
    fn start_port(&self, idx: usize) {
        let ptr = addr_of_mut!(self.hba().port_regs[idx].cmd);
        unsafe {
            while (ptr.read_volatile() & HBA_PXCMD_CR) > 0 {}
            ptr.write_volatile(ptr.read_volatile() | HBA_PXCMD_FRE);
            ptr.write_volatile(ptr.read_volatile() | HBA_PXCMD_ST);
        }
    }
    fn rebase(&self, port: &AHCIPort) {
        self.stop_port(port.number);
        unsafe {
            let page_addr = allocate_anywhere(command_list_base(self.number, port.number), 1024, NO_CACHE);
            addr_of_mut!(port.registers().clb).write_volatile(page_addr.translate() as u32);
            addr_of_mut!(port.registers().clbu).write_volatile(0);
            let page_addr = allocate_anywhere(fis_base(self.number, port.number), 256, NO_CACHE);
            addr_of_mut!(port.registers().fb).write_volatile(page_addr.translate() as u32);
            addr_of_mut!(port.registers().fbu).write_volatile(0);
            let hba_cmd_hdr = command_list_base(self.number, port.number) as *mut CommandHeader;
            for cmd_idx in 0..32 {
                let ptr = addr_of_mut!((*(hba_cmd_hdr.offset(cmd_idx))).prdtl);
                ptr.write_volatile(8);
                let ctba = command_table_descriptor(self.number, port.number, cmd_idx as usize);
                let page_addr = allocate_anywhere(ctba, 256, NO_CACHE);
                let ptr = addr_of_mut!((*(hba_cmd_hdr.offset(cmd_idx))).ctba);
                ptr.write_volatile(page_addr.translate() as u32);
                let ptr = addr_of_mut!((*(hba_cmd_hdr.offset(cmd_idx))).ctbau);
                ptr.write_volatile(0);
            }
        }
        self.start_port(port.number);
    }
    pub fn finish_initialisation(&mut self) {
        for port in &mut self.ports {
            port.finish_initialisation()
        }
    }
    pub fn handle_interrupt(&mut self) -> bool {
        let outstanding_irqs = self.hba().control_regs.is & self.taken_ports;
        if outstanding_irqs == 0 {
            return false;
        }
        let mut port_idx = 0;
        for idx in 0..32 {
            if (outstanding_irqs & (1 << idx)) > 0 {
                let port = &mut self.ports[port_idx];
                port.handle_interrupt();
                port_idx += 1;
            }
        }
        true
    }
}
