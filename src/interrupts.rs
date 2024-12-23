use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::println;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(exception_breakpoint_handler);
        idt
    };
}

pub fn create_idt() {
    IDT.load();
}

extern "x86-interrupt" fn exception_breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT OCCURED\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint() {
    x86_64::instructions::interrupts::int3();
}
