#![no_std] // Sem biblioteca padrão - bare metal
#![no_main] // Sem função main() padrão

use core::panic::PanicInfo;

// Constantes para cabeçalho Multiboot
const MAGIC: u32 = 0x1BADB002; // Número mágico Multiboot
const FLAGS: u32 = 0; // Flags de configuração
const CHECKSUM: u32 = 0u32.wrapping_sub(MAGIC).wrapping_sub(FLAGS); // Checksum para validação

// Estrutura do cabeçalho Multiboot
#[repr(C)] // Layout compatível com C
#[repr(align(4))] // Alinhamento de 4 bytes
struct MultibootHeader {
    magic: u32,
    flags: u32,
    checksum: u32,
}

// Cabeçalho Multiboot na seção especial
#[used] // Força inclusão no binário final
#[no_mangle] // Não alterar nome no linking
#[link_section = ".multiboot_header"] // Seção específica no binário
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: MAGIC,
    flags: FLAGS,
    checksum: CHECKSUM,
};

// Ponto de entrada do kernel
#[no_mangle] // Manter nome _start inalterado
pub extern "C" fn _start() -> ! {
    // Chamada externa C, nunca retorna
    // Limpa a primeira linha da tela
    let vga_buffer = 0xb8000 as *mut u8; // Endereço do buffer VGA
    for i in 0..80 {
        // 80 caracteres por linha
        unsafe {
            *vga_buffer.offset(i * 2) = b' '; // Caractere espaço
            *vga_buffer.offset(i * 2 + 1) = 0x07; // Atributo: cinza claro sobre preto
        }
    }

    // Escreve mensagem principal
    let hello = b"*** MICROKERNEL RUST FUNCIONANDO! ***";
    let mut i = 0;
    for &byte in hello.iter() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte; // Caractere
            *vga_buffer.offset(i as isize * 2 + 1) = 0x0F; // Atributo: branco sobre preto
        }
        i += 1;
    }

    // Adiciona segunda linha com instruções
    let line2 = b"Pressione Ctrl+Alt+G para sair do QEMU";
    let second_line_offset = 80 * 2; // Offset para segunda linha (80 chars * 2 bytes)
    for (i, &byte) in line2.iter().enumerate() {
        unsafe {
            *vga_buffer.offset((second_line_offset + i * 2) as isize) = byte; // Caractere
            *vga_buffer.offset((second_line_offset + i * 2 + 1) as isize) = 0x0E;
            // Atributo: amarelo sobre preto
        }
    }

    loop {} // Loop infinito - mantém kernel rodando
}

// Tratador de pânico personalizado
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Recebe info do pânico, nunca retorna
    loop {} // Loop infinito em caso de pânico
}
