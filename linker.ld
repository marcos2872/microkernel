/* Script do linker para o microkernel
 * Define o layout de memória e seções do kernel
 */

/* Ponto de entrada do kernel - função _start */
ENTRY(_start)

SECTIONS
{
    /* Carrega o kernel no endereço 1MB (0x100000)
     * Este é o endereço padrão para kernels Multiboot */
    . = 1M;

    /* Seção de boot - contém o cabeçalho Multiboot
     * Deve estar no início do binário para ser detectado pelo bootloader */
    .boot ALIGN(4) :
    {
        /* Garante que o cabeçalho multiboot esteja no início */
        *(.multiboot_header)
        . = ALIGN(4);
    }

    /* Seção de código executável
     * Contém todas as instruções do programa */
    .text :
    {
        *(.text)
    }

    /* Seção de dados somente leitura
     * Contém strings literais e outras constantes */
    .rodata :
    {
        *(.rodata)
    }

    /* Seção de dados inicializados
     * Contém variáveis globais com valores iniciais */
    .data :
    {
        *(.data)
    }

    /* Seção de dados não inicializados
     * Contém variáveis globais sem valores iniciais (zeradas) */
    .bss :
    {
        *(.bss)
    }
}
