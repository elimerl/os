#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(abi_x86_interrupt, alloc_error_handler)]

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

extern crate alloc;

mod allocator;
mod gdt;
mod interrupts;
mod memory;
mod writer;
use ::vga::vga;
use alloc::{boxed::Box, vec};
use bootloader::BootInfo;
use core::panic::PanicInfo;
use x86_64::{structures::paging::Page, VirtAddr};

use crate::memory::BootInfoFrameAllocator;
/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    hlt_loop();
}

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    init();
    println!("Hello from os code!");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    let x = Box::new(41);
    println!("heap_value at {:p}", x);
    let vec = vec![1, 2, 3, 4, 5];
    println!("vec at {:p}", vec.as_ptr());
    println!("If you see this text the kernel ran without crashing");
    hlt_loop();
}
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
pub fn init() {
    {
        let mut vga = vga::VGA.lock();
        vga.set_video_mode(vga::VideoMode::Mode80x25);
    }
    gdt::init();

    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }; // new
    x86_64::instructions::interrupts::enable(); // new
}
