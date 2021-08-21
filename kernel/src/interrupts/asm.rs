macro_rules! handler {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("push rax
                      push rcx
                      push rdx
                      push rsi
                      push rdi
                      push r8
                      push r9
                      push r10
                      push r11
                      mov rdi, rsp
                      add rdi, 9*8
                      call {}
                      pop r11
                      pop r10
                      pop r9
                      pop r8
                      pop rdi
                      pop rsi
                      pop rdx
                      pop rcx
                      pop rax
                      iretq", 
                      sym $name, options(noreturn));
            }
        }
        wrapper
    }}
}

macro_rules! handler_with_error_code {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("push rax
                      push rcx
                      push rdx
                      push rsi
                      push rdi
                      push r8
                      push r9
                      push r10
                      push r11
                      mov rsi, [rsp + 9*8] // load error code into rsi
                      mov rdi, rsp
                      add rdi, 10*8 // calculate exception stack frame pointer
                      sub rsp, 8 // align the stack pointer
                      call {}
                      add rsp, 8
                      pop r11
                      pop r10
                      pop r9
                      pop r8
                      pop rdi
                      pop rsi
                      pop rdx
                      pop rcx
                      pop rax
                      add rsp, 8 // pop error code
                      iretq", 
                      sym $name, options(noreturn));
            }
        }
        wrapper
    }}
}