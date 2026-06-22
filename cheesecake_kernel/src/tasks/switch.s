# Ah sh*t, here we go again...
.att_syntax prefix
.global context_switch
.type context_switch, @function

context_switch:
    # Only save old RSP if old_rsp ptr is non-null (kernel dummy has null).
    test %rdi, %rdi
    jz .load_new
    pushq %rbp
    pushq %rbx
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    movq %rsp, (%rdi)
    jmp .restore
.load_new:
    # No save needed — just switch to new stack.
    # We still need to align/setup for the pop sequence below,
    # but new task already has its context on stack.
.restore:
    movq %rsi, %rsp
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbx
    popq %rbp
    ret
