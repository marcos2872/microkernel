//! O ponto de entrada principal e o loop do kernel.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod gdt;
mod vga_buffer;
mod interrupts;
mod allocator;
mod memory;
mod task;
mod sync;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::structures::paging::{Page, Mapper};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::task::Scheduler;
use crate::allocator::HEAP_SIZE;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::collections::BTreeMap;

lazy_static! {
    /// O scheduler global.
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

lazy_static! {
    /// O registro de tarefas global.
    ///
    /// Mapeia um nome de serviço para um `TaskId`.
    static ref TASK_REGISTRY: Mutex<BTreeMap<String, task::TaskId>> =
        Mutex::new(BTreeMap::new());
}

lazy_static! {
    /// Um semáforo para proteger o acesso à tela.
    static ref SCREEN_SEMAPHORE: sync::Semaphore = sync::Semaphore::new(1);
}

use alloc::string::ToString;

fn ping_entry() -> ! {
    let pong_id = {
        let registry = TASK_REGISTRY.lock();
        *registry.get("pong").unwrap()
    };

    task::send(pong_id, 42);

    let reply = task::receive();
    SCREEN_SEMAPHORE.down();
    println!("ping: received reply: {}", reply);
    SCREEN_SEMAPHORE.up();

    loop {}
}

fn pong_entry() -> ! {
    let msg = task::receive();
    SCREEN_SEMAPHORE.down();
    println!("pong: received message: {}", msg);
    SCREEN_SEMAPHORE.up();

    let ping_id = {
        let registry = TASK_REGISTRY.lock();
        *registry.get("ping").unwrap()
    };
    task::send(ping_id, msg + 1);

    loop {}
}

/// Handler chamado em caso de erro de alocação de memória.
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

/// Handler chamado em caso de pânico.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

use x86_64::structures::paging::mapper::MapToError;
/// Inicializa o heap.
///
/// Mapeia as páginas do heap para frames físicos.
pub fn init_heap(
    mapper: &mut impl Mapper<x86_64::structures::paging::Size4KiB>,
    frame_allocator: &mut impl x86_64::structures::paging::FrameAllocator<x86_64::structures::paging::Size4KiB>,
) -> Result<(), MapToError<x86_64::structures::paging::Size4KiB>> {
    let page_range = {
        let heap_start = x86_64::VirtAddr::new(allocator::HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = x86_64::structures::paging::PageTableFlags::PRESENT | x86_64::structures::paging::PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        crate::allocator::ALLOCATOR.lock().init(allocator::HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

entry_point!(kernel_main);

/// O ponto de entrada principal do kernel.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    gdt::init();
    crate::vga_buffer::disable_cursor();
    clear_screen!();
    println!("Welcome to the microkernel!");
    println!("This is a basic implementation in Rust.");

    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    use alloc::vec;
    use task::Task;
    use x86_64::VirtAddr;

    let p4_table = unsafe { memory::active_level_4_table(phys_mem_offset) };

    let ping_stack = {
        const STACK_SIZE: usize = 4096;
        let stack_allocation = vec![0; STACK_SIZE].into_boxed_slice();
        let stack_ptr = Box::into_raw(stack_allocation) as *mut u8;
        let stack_top = unsafe { VirtAddr::from_ptr(stack_ptr.add(STACK_SIZE)) };
        stack_top
    };
    let ping_task = Task::new(VirtAddr::new(ping_entry as u64), ping_stack, p4_table);
    let ping_id = ping_task.id;

    let pong_stack = {
        const STACK_SIZE: usize = 4096;
        let stack_allocation = vec![0; STACK_SIZE].into_boxed_slice();
        let stack_ptr = Box::into_raw(stack_allocation) as *mut u8;
        let stack_top = unsafe { VirtAddr::from_ptr(stack_ptr.add(STACK_SIZE)) };
        stack_top
    };
    let pong_task = Task::new(VirtAddr::new(pong_entry as u64), pong_stack, p4_table);
    let pong_id = pong_task.id;

    let mut scheduler = SCHEDULER.lock();
    scheduler.add_task(ping_task);
    scheduler.add_task(pong_task);

    let mut registry = TASK_REGISTRY.lock();
    registry.insert("ping".to_string(), ping_id);
    registry.insert("pong".to_string(), pong_id);

    let mut current_task_context = task::TaskContext {
        rsp: VirtAddr::new(0),
        rbp: VirtAddr::new(0),
        rbx: 0,
        r12: 0,
        r13: 0,
        r14: 0,
        r15: 0,
        rip: VirtAddr::new(0),
    };

    let next_task_context = {
        let (_current_context, next_context) = scheduler.schedule().unwrap();
        next_context as *const _
    };
    drop(scheduler);

    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    unsafe {
        task::context_switch(&mut current_task_context, next_task_context);
    }

    loop {}
}
