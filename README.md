# Microkernel em Rust

Este é um projeto de um microkernel básico escrito em Rust, com o objetivo de demonstrar os princípios fundamentais de um sistema operacional minimalista. O projeto foi desenvolvido de forma incremental, adicionando funcionalidades passo a passo.

## Funcionalidades Implementadas

*   **Boot Bare-Metal:** O kernel é executado sem depender de um sistema operacional subjacente (`#![no_std]`).
*   **Saída de Vídeo:** Implementação de um driver para o modo de texto VGA, permitindo a escrita de texto e caracteres na tela. Inclui macros `println!` e `print!` para facilitar o uso.
*   **Tratamento de Interrupções:**
    *   Configuração de uma IDT (Interrupt Descriptor Table).
    *   Handlers para exceções da CPU (e.g., Page Fault, Division by Zero, Double Fault).
    *   Handlers para interrupções de hardware (timer e teclado).
*   **Driver de Teclado:** Suporte básico para teclado PS/2, permitindo a leitura de scancodes e a exibição de caracteres na tela.
*   **Gerenciamento de Memória:**
    *   Implementação de paginação para mapear a memória virtual para a física.
    *   Um alocador de frames para gerenciar a memória física.
    *   Um alocador de heap (`linked_list_allocator`) para alocação dinâmica.
*   **Scheduler de Tarefas:**
    *   Um scheduler round-robin simples para gerenciar tarefas.
    *   Troca de contexto implementada em assembly para alternar entre tarefas.
    *   Criação de duas tarefas de exemplo que executam concorrentemente.

## Estrutura do Projeto

O projeto é organizado nos seguintes módulos:

*   `main.rs`: Ponto de entrada do kernel, inicialização dos módulos e criação das tarefas.
*   `vga_buffer.rs`: Lida com a saída de texto no modo VGA.
*   `interrupts.rs`: Configura a IDT e os handlers de interrupção.
*   `memory.rs`: Gerencia a paginação e o alocador de frames.
*   `allocator.rs`: Define o alocador de heap global.
*   `task/`: Contém a lógica para gerenciamento de tarefas, incluindo o scheduler e a troca de contexto.
    *   `mod.rs`: Define as estruturas `Task` e `Scheduler`.
    *   `context.s`: Código assembly para a troca de contexto.

## Como Compilar e Executar

### Pré-requisitos

1.  **Rust Nightly:** É necessário ter o toolchain `nightly` do Rust.
    ```sh
    rustup toolchain install nightly
    rustup override set nightly
    ```
2.  **Componentes do Rust:** Instale os componentes `rust-src` e `llvm-tools-preview`.
    ```sh
    rustup component add rust-src
    rustup component add llvm-tools-preview
    ```
3.  **Bootimage:** Instale a ferramenta `bootimage` para criar a imagem de boot.
    ```sh
    cargo install bootimage
    ```
4.  **QEMU:** É necessário ter o QEMU instalado para executar a imagem do kernel.

### Compilação

Para compilar o kernel e criar a imagem de boot, execute o seguinte comando na raiz do projeto:

```sh
cargo bootimage
```

Este comando irá gerar um arquivo de imagem de boot em `target/x86_64-blog_os/debug/bootimage-microkernel.bin`.

### Execução

Para executar o kernel no QEMU, use o seguinte comando:

```sh
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-microkernel.bin
```

Você deverá ver a mensagem de boas-vindas, seguida por uma alternância de "1" e "2" sendo impressos na tela (das duas tarefas de exemplo), e poderá digitar no teclado para ver os caracteres aparecerem.
