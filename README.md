# Microkernel em Rust

Um microkernel simples implementado em Rust que exibe uma mensagem na tela usando acesso direto ao buffer VGA.

## Descrição

Este projeto implementa um microkernel básico que:
- Roda em bare metal (sem sistema operacional)
- Usa bootloader Multiboot manual (sem dependências externas)
- Escreve diretamente no buffer VGA (0xb8000) 
- Exibe a mensagem "Ola! Este e meu microkernel em Rust!" na tela

## Pré-requisitos

- Rust (versão 1.60 ou superior)
- Cargo
- QEMU (para executar o kernel)

### Instalação das dependências

1. **Instalar Rust e Cargo:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Adicionar target para bare metal:**
   ```bash
   rustup target add x86_64-unknown-none
   ```

3. **Instalar QEMU:**
   ```bash
   # Ubuntu/Debian
   sudo apt install qemu-system-x86

   # Arch Linux
   sudo pacman -S qemu

   # macOS
   brew install qemu
   ```

## Como compilar

```bash
cargo build --release
```

Para verificar se compila corretamente:
```bash
cargo check
```

## Como executar

### Método 1: Imagem de Disco HDD (Recomendado)

1. **Compilar o kernel:**
   ```bash
   cargo build --release
   ```

2. **Criar imagem de disco:**
   ```bash
   qemu-img create -f raw disk.img 50M
   ```

3. **Configurar partição e filesystem:**
   ```bash
   sudo losetup -P /dev/loop0 disk.img
   echo -e "n\np\n1\n\n\na\nw\n" | sudo fdisk /dev/loop0
   sudo partprobe /dev/loop0
   sudo mkfs.ext2 /dev/loop0p1
   ```

4. **Montar e copiar arquivos:**
   ```bash
   sudo mkdir -p /mnt/bootdisk
   sudo mount /dev/loop0p1 /mnt/bootdisk
   sudo mkdir -p /mnt/bootdisk/boot/grub
   sudo cp target/x86_64-unknown-none/release/microkernel /mnt/bootdisk/boot/
   ```

5. **Criar configuração do GRUB:**
   ```bash
   echo -e "set timeout=1\nset default=0\n\nmenuentry \"Microkernel Rust\" {\n    multiboot /boot/microkernel\n    boot\n}" | sudo tee /mnt/bootdisk/boot/grub/grub.cfg
   ```

6. **Instalar GRUB:**
   ```bash
   sudo grub-install --target=i386-pc --boot-directory=/mnt/bootdisk/boot /dev/loop0
   ```

7. **Finalizar:**
   ```bash
   sudo umount /mnt/bootdisk
   sudo losetup -d /dev/loop0
   ```

8. **Executar no QEMU:**
   ```bash
   qemu-system-x86_64 -drive file=disk.img,format=raw -nic none
   ```

### Método 2: Imagem ISO (Alternativo)

1. **Compilar e preparar estrutura:**
   ```bash
   cargo build --release
   mkdir -p iso/boot/grub
   cp target/x86_64-unknown-none/release/microkernel iso/boot/
   ```

2. **Gerar imagem ISO:**
   ```bash
   grub-mkrescue -o microkernel.iso iso/
   ```

3. **Executar (pode ter problemas de compatibilidade):**
   ```bash
   qemu-system-x86_64 -cdrom microkernel.iso -boot d,strict=on
   ```

### Script Automatizado

Para simplificar o processo, você pode usar este script:

```bash
#!/bin/bash
# build-and-run.sh

# Compila o kernel
cargo build --release

# Cria e configura a imagem de disco (se não existir)
if [ ! -f disk.img ]; then
    echo "Criando imagem de disco..."
    qemu-img create -f raw disk.img 50M
    
    sudo losetup -P /dev/loop0 disk.img
    echo -e "n\np\n1\n\n\na\nw\n" | sudo fdisk /dev/loop0
    sudo partprobe /dev/loop0
    sudo mkfs.ext2 /dev/loop0p1
    
    sudo mkdir -p /mnt/bootdisk
    sudo mount /dev/loop0p1 /mnt/bootdisk
    sudo mkdir -p /mnt/bootdisk/boot/grub
    
    echo -e "set timeout=1\nset default=0\n\nmenuentry \"Microkernel Rust\" {\n    multiboot /boot/microkernel\n    boot\n}" | sudo tee /mnt/bootdisk/boot/grub/grub.cfg
    
    sudo grub-install --target=i386-pc --boot-directory=/mnt/bootdisk/boot /dev/loop0
    sudo umount /mnt/bootdisk
    sudo losetup -d /dev/loop0
fi

# Atualiza o kernel na imagem existente
sudo losetup -P /dev/loop0 disk.img
sudo mount /dev/loop0p1 /mnt/bootdisk
sudo cp target/x86_64-unknown-none/release/microkernel /mnt/bootdisk/boot/
sudo umount /mnt/bootdisk
sudo losetup -d /dev/loop0

# Executa no QEMU
echo "Iniciando microkernel..."
qemu-system-x86_64 -drive file=disk.img,format=raw -nic none
```

### Dependências necessárias

```bash
# Ubuntu/Debian
sudo apt install qemu-system-x86 grub-pc-bin

# Arch Linux
sudo pacman -S qemu-base grub
```

## Estrutura do projeto

```
.
├── Cargo.toml              # Configuração do projeto (sem dependências externas)
├── .cargo/config.toml      # Configuração do target e linker
├── linker.ld               # Script do linker para layout de memória
├── x86_64-unknown-none.json # Especificação do target bare metal
├── disk.img                # Imagem de disco HDD bootável (50MB)
├── microkernel.iso         # Imagem ISO bootável (alternativa)
├── build-and-run.sh        # Script automatizado para build e execução
├── iso/                    # Estrutura para criação da ISO
│   └── boot/
│       ├── grub/
│       │   └── grub.cfg   # Configuração do bootloader GRUB para ISO
│       └── microkernel    # Binário do kernel (copiado após build)
├── src/
│   └── main.rs            # Código principal do microkernel com Multiboot header válido
└── README.md              # Este arquivo
```

## O que você verá ao executar

Quando o microkernel carrega, você verá na tela:

- **Primeira linha**: `*** MICROKERNEL RUST FUNCIONANDO! ***` (texto branco sobre fundo preto)
- **Segunda linha**: `Pressione Ctrl+Alt+G para sair do QEMU` (texto amarelo sobre fundo preto)

## Troubleshooting

### Problema: "Could not read from CDROM (code 0009)"
**Solução**: Use o método de imagem de disco HDD ao invés da ISO. Algumas versões do SeaBIOS/QEMU têm problemas com ISOs geradas pelo grub-mkrescue.

### Problema: "No bootable device"
**Solução**: 
1. Verifique se o kernel é reconhecido como Multiboot válido:
   ```bash
   grub-file --is-x86-multiboot target/x86_64-unknown-none/release/microkernel
   ```
2. Se não for válido, recompile o projeto.

### Problema: GRUB não encontra o kernel
**Solução**: Verifique se o arquivo está no caminho correto `/boot/microkernel` dentro da imagem.

### Problema: Permissões no loop device
**Solução**: Execute os comandos com `sudo` e certifique-se de que `/dev/loop0` está disponível.

## Funcionamento

O microkernel funciona da seguinte forma:

1. **Multiboot Header**: Cabeçalho compatível com GRUB/bootloaders Multiboot
2. **Entrada (`_start`)**: Ponto de entrada do kernel após o bootloader
3. **Buffer VGA**: Escreve diretamente no endereço de memória 0xb8000 (buffer de texto VGA)
4. **Formatação**: Cada caractere ocupa 2 bytes (caractere + atributo de cor 0x0f - branco sobre preto)
5. **Loop infinito**: Mantém o kernel rodando após exibir a mensagem

## Características técnicas

- **Linguagem**: Rust
- **Target**: x86_64-unknown-none (bare metal)
- **Bootloader**: Multiboot header manual (sem dependências externas)
- **Linker**: Script customizado para layout de memória
- **Panic handler**: Loop infinito personalizado
- **Modo VGA**: Texto 80x25 com 16 cores

## Vantagens desta implementação

- **Zero dependências externas**: Funciona offline sem precisar baixar crates
- **Compatível com GRUB**: Pode ser carregado por qualquer bootloader Multiboot
- **Simples e educativo**: Código fácil de entender para aprender OS development
- **Compilação rápida**: Sem dependências para baixar ou compilar

## Limitações

- Apenas exibe uma mensagem estática
- Não possui sistema de interrupções
- Não gerencia memória dinamicamente
- Não possui drivers de hardware