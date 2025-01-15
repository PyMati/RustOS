#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rustos::{allocator, println};
use x86_64::structures::paging::{PageTable, Translate};
use x86_64::VirtAddr;
extern crate alloc;
use alloc::boxed::Box;
use rustos::async_task::kb;
use rustos::async_task::{executor::Executor, executor::SimpleExecutor, Task};

use rustos::memory;
use rustos::memory::BootInfoFrameAllocator;
use x86_64::structures::paging::Page;

entry_point!(kernel_main);

fn random_double_fault() {
    // Testing purposes of double fault
    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    };
    // Testing purposes for single fault
    x86_64::instructions::interrupts::int3();
}

fn read_and_traverse_l4_table(boot_info: &'static BootInfo) {
    // Reading level 4 table from memory
    // TESTS
    use rustos::memory::active_level_4_table;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry {}: {:?}", i, entry);

            let phys = entry.frame().unwrap().start_address();
            let virt = phys.as_u64() + boot_info.physical_memory_offset;
            let ptr = VirtAddr::new(virt).as_mut_ptr();
            let l3_table: &PageTable = unsafe { &*ptr };

            // print non-empty entries of the level 3 table
            for (i, entry) in l3_table.iter().enumerate() {
                if !entry.is_unused() {
                    println!("  L3 Entry {}: {:?}", i, entry);
                }
            }
        }
    }
}

fn translate_adderesses(boot_info: &'static BootInfo) {
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    rustos::init();

    // Example allocator test

    // random_double_fault();
    // read_and_traverse_l4_table(boot_info);
    // translate_adderesses(boot_info);

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };
    let x = Box::new(4);
    println!("{:p}", x);

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(kb::print_keypresses()));
    executor.run();

    #[cfg(test)]
    test_main();

    rustos::hlt();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rustos::hlt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustos::test_panic_handler(info)
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}
