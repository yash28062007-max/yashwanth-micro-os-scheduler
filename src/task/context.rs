//! CPU Register Context and Hardware Task Switching
//!
//! Handles saving and restoring callee-preserved x86_64 registers during
//! task context switches. Uses stable global assembly (`global_asm!`) to ensure
//! compatibility with standard Rust compilers without requiring unstable compiler features.

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskContext {
    pub rsp: u64,
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub rip: u64,
    pub rflags: u64,
}

impl TaskContext {
    pub const fn empty() -> Self {
        Self {
            rsp: 0,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: 0,
            rflags: 0x202, // IF (Interrupt Enable) flag enabled by default
        }
    }

    /// Initializes a task stack and context so that when switched to, it starts at `entry_point`.
    pub fn init_stack(&mut self, stack_top: usize, entry_point: usize) {
        self.rsp = stack_top as u64;
        self.rbp = stack_top as u64;
        self.rip = entry_point as u64;
        self.rflags = 0x202;
    }
}

#[cfg(any(target_os = "none", feature = "kernel_bin"))]
core::arch::global_asm!(
    r#"
.global switch_context
.type switch_context, @function
switch_context:
    /* Save current task callee-saved registers into old_ctx (*rdi) */
    mov [rdi + 0x00], rsp
    mov [rdi + 0x08], r15
    mov [rdi + 0x10], r14
    mov [rdi + 0x18], r13
    mov [rdi + 0x20], r12
    mov [rdi + 0x28], rbx
    mov [rdi + 0x30], rbp
    lea rax, [rip + 1f]
    mov [rdi + 0x38], rax
    pushfq
    pop rax
    mov [rdi + 0x40], rax

    /* Restore next task callee-saved registers from new_ctx (*rsi) */
    mov rax, [rsi + 0x40]
    push rax
    popfq
    mov rbp, [rsi + 0x30]
    mov rbx, [rsi + 0x28]
    mov r12, [rsi + 0x20]
    mov r13, [rsi + 0x18]
    mov r14, [rsi + 0x10]
    mov r15, [rsi + 0x08]
    mov rsp, [rsi + 0x00]
    mov rax, [rsi + 0x38]
    jmp rax
1:
    ret
"#
);

#[cfg(any(target_os = "none", feature = "kernel_bin"))]
extern "C" {
    /// Low-level context switch between two TaskContext structs.
    ///
    /// # Safety
    /// `old_ctx` and `new_ctx` must be valid, 8-byte aligned pointers to `TaskContext`.
    pub fn switch_context(old_ctx: *mut TaskContext, new_ctx: *const TaskContext);
}

#[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
#[no_mangle]
pub unsafe extern "C" fn switch_context(old_ctx: *mut TaskContext, new_ctx: *const TaskContext) {
    let _ = (old_ctx, new_ctx);
}
