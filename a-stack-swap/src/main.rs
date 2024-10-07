#![allow(unused, dead_code)]
use core::arch::asm;

/// Size of stack
const SSIZE: isize = 48;

/// Represents our CPU state
#[cfg(all(target_arch = "x86_64", target_family = "unix"))]
#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    /// register that stores stack pointer
    rsp: u64,
}

fn hello() -> ! {
    println!("I WAKEUP ON A NEW STACK");
    loop {}
}

/// Function to switch to a new stack
///
/// Takes in a ThreadContext and updates the stack pointer
/// to point to the address stored within the ThreadContext.
///
/// Uses the `ret` intrusction to pop the value the rsp is
/// pointing to, into the rip registry (instruction pointer) and
/// hence on next instruction cycle it will load the next
/// instruction at the address stored in the instruction
/// pointer register.
#[cfg(all(target_arch = "x86_64", target_family = "unix"))]
unsafe fn gt_switch(new: *const ThreadContext) {
    // `in(reg) new let's compiler decide on general register to store value of new
    asm!(
        "mov rsp, [{0} + 0x00]",
        "ret",
        in(reg) new
    )
}

#[cfg(all(target_arch = "aarch64", target_family = "unix"))]
unsafe fn gt_switch(new: *const ThreadContext) {
    unimplemented!("not implemented for aarch64")
}

fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0_u8; SSIZE as usize];

    // Our stack grows downwards (a descending stack)
    // our 48 byte stack starts at index 0 and ends at index 47, index 31
    // will be the first index of a 16-byte offset from the base of our stack.
    //
    // We write pointer to an offset of 16 bytes from the base of our stack
    unsafe {
        // Get pointer to bottom / base of our stack
        let stack_bottom = stack.as_mut_ptr().offset(SSIZE);
        // round address down to nearest 16 byte address
        let sb_aligned = (stack_bottom as usize & !15) as *mut u8;

        // write address of hello to a 16 byte offset within stack
        std::ptr::write(sb_aligned.offset(-16) as *mut u64, hello as u64);

        // set stack pointer to this 16 byte offset within our stack.
        // This will be the new stack pointer for our context.
        ctx.rsp = sb_aligned.offset(-16) as u64;

        println!("\n\nFunction address: {:02x}\n\n", hello as u64);
        println!("Stack pointer address: {}\n\n", ctx.rsp);
        println!("Note: \n\n");
        println!(
            r#"Stack layout: 

Addresses are printed in descending order (high -> low)

The stack also grows downwards, meaning that pushing to the
stack will decrement the stack pointer (goes downwards in
address space) and popping from the stack will increment
the stack pointer (goes upwards in address space).

push -> rsp decremented
pop  -> rsp incremented

All offsets to addresses in the stack are reachable via
a positive offset from the stack pointer (rsp + offset).
"#
        );
        for (i, byte) in stack.iter().enumerate() {
            // println!("{}: {:02x} ", ((SSIZE - 1) as usize) - i, byte);
            // println!("{}: {:02x} ", i, byte);

            let mem = sb_aligned.offset(-(i as isize)) as usize;

            println!(
                "index: {:2}, offset: {:2}, mem: {}, val: {:02x} {}",
                SSIZE - 1 - (i as isize),
                mem % 16,
                mem,
                *sb_aligned.offset(-(i as isize)),
                if mem == ctx.rsp as usize {
                    format!("<---- Stack pointer address: {}", mem)
                } else if mem == sb_aligned as usize {
                    format!("<---- Stack base address: {}", mem)
                } else {
                    format!("")
                }
            );

            if i % 16 == 0 {
                println!("--------- {i} bytes --------------");
            }
        }
        gt_switch(&mut ctx);
    }
}
