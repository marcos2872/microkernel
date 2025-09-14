#![no_std]
#![no_main]

use core::panic::PanicInfo;

const MAGIC: u32 = 0x1BADB002;
const FLAGS: u32 = 0;
const CHECKSUM: u32 = 0u32.wrapping_sub(MAGIC).wrapping_sub(FLAGS);

#[repr(C)]
#[repr(align(4))]
struct MultibootHeader {
    magic: u32,
    flags: u32,
    checksum: u32,
}

#[used]
#[no_mangle]
#[link_section = ".multiboot_header"]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: MAGIC,
    flags: FLAGS,
    checksum: CHECKSUM,
};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Limpa a tela primeiro - preenche toda a primeira linha com espaços
    let vga_buffer = 0xb8000 as *mut u8;
    for i in 0..80 {
        unsafe {
            *vga_buffer.offset(i * 2) = b' ';
            *vga_buffer.offset(i * 2 + 1) = 0x07; // Cinza claro sobre preto
        }
    }

    // Escreve a mensagem
    let hello = b"*** MICROKERNEL RUST FUNCIONANDO! ***";
    let mut i = 0;
    for &byte in hello.iter() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0x0F; // Branco sobre preto
        }
        i += 1;
    }

    // Adiciona uma segunda linha para confirmar
    let line2 = b"Pressione Ctrl+Alt+G para sair do QEMU";
    let second_line_offset = 80 * 2; // Segunda linha
    for (i, &byte) in line2.iter().enumerate() {
        unsafe {
            *vga_buffer.offset((second_line_offset + i * 2) as isize) = byte;
            *vga_buffer.offset((second_line_offset + i * 2 + 1) as isize) = 0x0E;
            // Amarelo sobre preto
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
