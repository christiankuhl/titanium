use crate::{asm, idle};

#[macro_export]
macro_rules! handler {
    ($name: ident) => {{
        use core::arch::asm;
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("
                    push rax
                    push rbx
                    push rcx
                    push rdx
                    push rsi
                    push rdi
                    push r8
                    push r9
                    push r10
                    push r11
                    push r12
                    push r13
                    push r14
                    push r15
                    push rbp
                    mov rdi, rsp
                    add rdi, 15*8
                    mov rsi, rsp
                    call {}
                    mov rsp, rax
                    pop rbp
                    pop r15
                    pop r14
                    pop r13
                    pop r12
                    pop r11
                    pop r10
                    pop r9
                    pop r8
                    pop rdi
                    pop rsi
                    pop rdx
                    pop rcx
                    pop rbx
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
        use core::arch::asm;
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

#[naked]
pub unsafe extern "C" fn enter_userspace() -> ! {
    use core::arch::asm;
    asm!("
    mov rcx, 0xc0000082
	wrmsr
	mov rcx, 0xc0000080
	rdmsr
	or eax, 1
	wrmsr
	mov rcx, 0xc0000081
	rdmsr
	mov edx, 0x00180008
	wrmsr
	mov ecx, test_user_function
	mov r11, 0x202
	sysretq", options(noreturn));
}

#[no_mangle]
extern "C" fn test_user_function() {
    use core::arch::asm;
    unsafe { asm!("cli") }
}