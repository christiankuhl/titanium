use crate::asm::{inb, inl, inw, outb, outl, outw};
use core::marker::PhantomData;

pub struct Port<T: PortIO> {
    base: u16,
    phantom: PhantomData<T>,
}

impl<T> Port<T>
where
    T: PortIO,
{
    pub fn new(base: u16) -> Self {
        Self { base, phantom: PhantomData }
    }
    #[inline]
    pub unsafe fn write(&self, value: T) {
        T::write(self.base, value);
    }
    #[inline]
    pub unsafe fn read(&self) -> T {
        T::read(self.base)
    }
}

pub trait PortIO {
    unsafe fn write(port: u16, value: Self);
    unsafe fn read(port: u16) -> Self;
}

impl PortIO for u8 {
    #[inline]
    unsafe fn write(port: u16, value: u8) {
        outb(port, value);
    }
    #[inline]
    unsafe fn read(port: u16) -> u8 {
        inb(port)
    }
}

impl PortIO for u16 {
    #[inline]
    unsafe fn write(port: u16, value: u16) {
        outw(port, value);
    }
    #[inline]
    unsafe fn read(port: u16) -> u16 {
        inw(port)
    }
}

impl PortIO for u32 {
    #[inline]
    unsafe fn write(port: u16, value: u32) {
        outl(port, value);
    }
    #[inline]
    unsafe fn read(port: u16) -> u32 {
        inl(port)
    }
}
