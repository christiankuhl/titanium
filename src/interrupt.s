.set IRQ_BASE, 0x20

.section .text
.extern handle_interrupt

.macro handle_exception num
.global handle_exception_\num\()
    movb $\num + IRQ_BASE, (interruptnumber)
    jmp int_bottom
.endm

.macro handle_interrupt_request num
.global handle_interrupt_request_\num\()
    movb $\num, (interruptnumber)
    jmp int_bottom
.endm

handle_interrupt_request 0x00
handle_interrupt_request 0x01

int_bottom:
    pushq %r15
    pushq %r14
    pushq %r13
    pushq %r12
    pushq %r11
    pushq %r10
    pushq %r9
    pushq %r8
    pushq %rax
    pushq %rcx
    pushq %rdx
    pushq %rbx
    pushq %rsp
    pushq %rbp
    pushq %rsi
    pushq %rdi
    push (interruptnumber)
    call handle_interrupt
    movl %eax, %esp
    popq %rdi
    popq %rsi
    popq %rbp
    popq %rsp
    popq %rbx
    popq %rdx
    popq %rcx
    popq %rax
    popq %r8
    popq %r9
    popq %r10
    popq %r11
    popq %r12
    popq %r13
    popq %r14
    popq %r15
    iret

.idata:
    interruptnumber: .byte 0
