
use crate::interrupts::{SegmentSelector, DescriptorTablePointer};

const EFER: u32 = 0xC000_0080;
const NXE_BIT: u32 = 1 << 11;
const WRITE_PROTECT: usize = 1 << 16;

pub struct Cr3;

impl Cr3 {
    #[inline]
    pub fn read() -> usize {
        let value: usize;
        unsafe {
            asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
        }
        value
    }
    #[inline]
    pub fn p4_address() -> usize {
        Cr3::read() &  0x_000f_ffff_ffff_f000
    }
    #[inline]
    pub unsafe fn write(value: usize) {
        asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
    }
}

#[inline]
pub unsafe fn enable_nxe_bit() {
    let (high, low): (u32, u32);
    asm!(
        "rdmsr",
        in("ecx") EFER,
        out("eax") low, out("edx") high,
        options(nomem, nostack, preserves_flags),
    );
    asm!(
        "wrmsr",
        in("ecx") EFER,
        in("eax") low | NXE_BIT, in("edx") high,
        options(nostack, preserves_flags),
    );
}

#[inline]
pub unsafe fn enable_write_protect_bit() {
    let value: usize;
    asm!("mov {}, cr0", out(reg) value, options(nomem, nostack, preserves_flags));
    asm!("mov cr0, {}", in(reg) value | WRITE_PROTECT, options(nostack, preserves_flags));
}

pub fn idle() -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[inline]
pub fn code_segment_selector() -> SegmentSelector {
    let segment: u16;
    unsafe { asm!("mov {}, cs", out(reg) segment, options(nomem, nostack, preserves_flags)); };
    SegmentSelector(segment)
}

#[inline]
pub unsafe fn load_interrupt_descriptor_table(idt: &DescriptorTablePointer) {
    asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
}