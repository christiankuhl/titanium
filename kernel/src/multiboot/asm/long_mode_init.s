.intel_syntax noprefix
.global long_mode_start
.extern kernel_main

.section .text
.code64
long_mode_start:
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    call kernel_main

    # print `OKAY` to screen
    mov rax, 0x2f592f412f4b2f4f
    mov qword [0xb8000], rax
    hlt
