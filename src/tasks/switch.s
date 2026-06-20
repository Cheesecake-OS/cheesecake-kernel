# Ah sh*t, here we go again...
.att_syntax prefix
.global context_switch
.type context_switch, @function

context_switch:
    # Save the old task callee-saved registers to its stack
    pushq %rbp
    pushq %rbx
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    # Save the old task stack pointer (%rsp) to the address passed in %rdi
    movq %rsp, (%rdi)

    # Switch to the new task stack pointer by loading %rsi to %rsp
    movq %rsi, %rsp

    # Restore the new task callee-saved registers from its stack
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbx
    popq %rbp

    # Jump to the new task's RIP (top of the stack now)
    ret
