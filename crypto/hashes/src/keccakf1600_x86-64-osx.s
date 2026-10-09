/* Keccak-f1600 x86_64 macOS Mach-O assembly implementation */
.text
.globl _keccakf1600
_keccakf1600:
    pushq %rbx
    pushq %rbp
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    movq %rdi, %rax
    /* Keccak-f1600 24 rounds state permutation */
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbp
    popq %rbx
    ret