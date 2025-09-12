//! Este módulo lida com o tratamento de interrupções e exceções da CPU.

use crate::{print, println};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use core::convert::From;
use core::result::Result::Ok;
use core::option::Option::Some;
use core::panic;

/// O offset inicial para as interrupções do PIC primário.
pub const PIC_1_OFFSET: u8 = 32;
/// O offset inicial para as interrupções do PIC secundário.
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// O controlador de interrupções programável (PIC) global.
///
/// É protegido por um `Mutex` para garantir o acesso seguro de múltiplos contextos.
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Enumeração dos índices de interrupção de hardware.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    /// Interrupção do timer (PIT).
    Timer = PIC_1_OFFSET,
    /// Interrupção do teclado.
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

use crate::gdt;

lazy_static! {
    /// A Tabela de Descritores de Interrupção (IDT) global.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.divide_error.set_handler_fn(division_by_zero_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

/// Inicializa a IDT.
///
/// Carrega a IDT no registrador da CPU para que as interrupções possam ser tratadas.
pub fn init_idt() {
    IDT.load();
}

lazy_static! {
    /// O driver de teclado global.
    ///
    /// É protegido por um `Mutex` para lidar com o acesso concorrente.
    static ref KEYBOARD: spin::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        spin::Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
            HandleControl::Ignore)
        );
}

/// Handler para a interrupção do teclado.
///
/// Lê o scancode da porta do teclado, o decodifica e o imprime na tela.
/// Envia um sinal de "End of Interrupt" (EOI) ao PIC.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

/// Handler para a exceção de breakpoint.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Handler para a exceção de page fault.
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop {}
}

/// Handler para a exceção de divisão por zero.
extern "x86-interrupt" fn division_by_zero_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DIVISION BY ZERO\n{:#?}", stack_frame);
    loop {}
}

/// Handler para a exceção de double fault.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// Handler para a interrupção do timer.
///
/// Aciona o scheduler para realizar a troca de contexto.
/// Envia um sinal de "End of Interrupt" (EOI) ao PIC.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use crate::task::context_switch;
    use crate::SCHEDULER;

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }

    let mut scheduler = SCHEDULER.lock();
    if let Some((current_context, next_context)) = scheduler.schedule() {
        let current_context_ptr = current_context as *mut _;
        let next_context_ptr = next_context as *const _;
        drop(scheduler);
        unsafe {
            context_switch(current_context_ptr, next_context_ptr);
        }
    }
}
