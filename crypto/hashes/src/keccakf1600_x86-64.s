/* Keccak-f1600 x86_64 System V ABI assembly implementation */
.text
.globl keccakf1600
.type keccakf1600, @function
keccakf1600:
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