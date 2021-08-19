use crate::interrupts::{DescriptorTablePointer, SegmentSelector};

const EFER: u32 = 0xc000_0080;
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
        Cr3::read() & 0x_000f_ffff_ffff_f000
    }
    #[inline]
    pub unsafe fn write(value: usize) {
        asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
    }
}

/// Invalidate the TLB completely by reloading the CR3 register.
#[inline]
pub fn tlb_flush_all() {
    let value = Cr3::read();
    unsafe { Cr3::write(value) }
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
    unsafe {
        asm!("mov {}, cs", out(reg) segment, options(nomem, nostack, preserves_flags));
    };
    SegmentSelector(segment)
}

pub unsafe fn set_code_segment_selector(sel: SegmentSelector) {
    asm!(
        "push {sel}",
        "lea {tmp}, [1f + rip]",
        "push {tmp}",
        "retfq",
        "1:",
        sel = in(reg) u64::from(sel.0),
        tmp = lateout(reg) _,
        options(preserves_flags),
    );
}


#[inline]
pub unsafe fn load_interrupt_descriptor_table(idt: &DescriptorTablePointer) {
    asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
}

#[inline]
pub unsafe fn load_global_descriptor_table(idt: &DescriptorTablePointer) {
    asm!("lgdt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
}

#[inline]
pub unsafe fn load_task_state_segment(sel: SegmentSelector) {
    asm!("ltr {0:x}", in(reg) sel.0, options(nomem, nostack, preserves_flags));
}

#[inline]
pub fn breakpoint() {
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}

#[inline]
pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

#[inline]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

#[inline]
pub fn interrupts_enabled() -> bool {
    let r: u64;
    unsafe {
        asm!("pushfq; pop {}", out(reg) r, options(nomem, preserves_flags));
    }
    r & (1 << 9) > 0
}

#[inline]
pub fn page_fault_linear_address() -> usize {
    let value: usize;
    unsafe {
        asm!("mov {}, cr2", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

#[inline]
pub fn without_interrupts<F, R>(f: F) -> R
    where F: FnOnce() -> R,
{
    let enabled_before = interrupts_enabled();
    if enabled_before {
        disable_interrupts();
    }
    let result = f();
    if enabled_before {
        enable_interrupts();
    }
    result
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[inline]
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[inline]
pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn outw(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn outl(port: u16, value: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
}

#[inline]
pub fn flush_tlb(addr: u64) {
    unsafe {
        asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
}