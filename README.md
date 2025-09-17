# Microkernel em Rust

Um microkernel simples implementado em Rust que exibe mensagens na tela usando acesso direto ao buffer VGA.

## 🎯 Descrição

Este projeto implementa um microkernel básico que:
- Roda em bare metal (sem sistema operacional)
- Usa bootloader Multiboot manual (sem dependências externas)
- Escreve diretamente no buffer VGA (0xb8000)
- Exibe mensagens coloridas na tela

## 🔧 Pré-requisitos

### Ferramentas básicas
- **Rust** (versão 1.60 ou superior)
- **Cargo**
- **QEMU** (para executar o kernel)

### Instalação rápida

1. **Instalar Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   rustup target add x86_64-unknown-none
   ```

2. **Instalar QEMU e dependências:**
   ```bash
   # Ubuntu/Debian
   sudo apt install qemu-system-x86 grub-pc-bin xorriso mtools
   
   # Arch Linux
   sudo pacman -S qemu-base grub libisoburn mtools
   
   # Fedora/CentOS
   sudo dnf install qemu-system-x86 grub2-tools xorriso mtools
   ```

## 🚀 Execução Rápida

### Método Recomendado: Script Automatizado

```bash
chmod +x build-and-run.sh
./build-and-run.sh
```

**O que o script faz:**
- ✅ Compila o kernel automaticamente
- ✅ Cria/atualiza imagem de disco bootável
- ✅ Verifica se é Multiboot válido
- ✅ Resolve problemas de dispositivos loop ocupados
- ✅ Inicia o QEMU

**Para sair do QEMU:** `Ctrl+Alt+G` ou feche a janela

## 📋 Métodos de Execução Detalhados

### 1. Imagem de Disco HDD (Recomendado)

```bash
# Compilar
cargo build --release

# Executar com script (mais fácil)
./build-and-run.sh

# OU executar manualmente
qemu-system-x86_64 -drive file=disk.img,format=raw -nic none
```

### 2. Imagem ISO (Alternativo)

```bash
# Preparar estrutura
cargo build --release
mkdir -p iso/boot/grub
cp target/x86_64-unknown-none/release/microkernel iso/boot/

# Gerar ISO
grub-mkrescue -o microkernel.iso iso/

# Executar
qemu-system-x86_64 -cdrom microkernel.iso
```

**Possíveis erros na geração da ISO:**
- `xorriso not found` → `sudo apt install xorriso`
- `mformat invocation failed` → `sudo apt install mtools`

### 3. Outras formas de usar a ISO

**VirtualBox:**
- Crie uma nova VM e use `microkernel.iso` como CD/DVD

**USB bootável:**
```bash
sudo dd if=microkernel.iso of=/dev/sdX bs=4M status=progress
```

**Verificar conteúdo:**
```bash
sudo mount -o loop microkernel.iso /mnt && ls -la /mnt/boot && sudo umount /mnt
```

## 📂 Estrutura do Projeto

```
microkernel/
├── src/main.rs                 # Código principal do kernel
├── Cargo.toml                  # Configuração do projeto
├── .cargo/config.toml          # Configuração do target
├── linker.ld                   # Script do linker
├── x86_64-unknown-none.json    # Target bare metal
├── build-and-run.sh            # Script automatizado ⭐
├── disk.img                    # Imagem HDD (criada automaticamente)
├── microkernel.iso             # Imagem ISO (opcional)
└── iso/boot/grub/grub.cfg      # Config GRUB para ISO
```

## 💻 O que você verá

Quando o microkernel carrega:
- **Linha 1:** `*** MICROKERNEL RUST FUNCIONANDO! ***` (branco)
- **Linha 2:** `Pressione Ctrl+Alt+G para sair do QEMU` (amarelo)

## 🔍 Troubleshooting

| Problema | Solução |
|----------|---------|
| `Could not read from CDROM` | Use imagem HDD em vez de ISO |
| `No bootable device` | Verifique se o kernel é Multiboot válido:<br>`grub-file --is-x86-multiboot target/x86_64-unknown-none/release/microkernel` |
| `GRUB não encontra kernel` | Verifique se está em `/boot/microkernel` |
| `Device or resource busy` | Script resolve automaticamente |
| `Permission denied` | Use `sudo` nos comandos de montagem |

## 🏗️ Como Funciona

1. **Multiboot Header** → Compatível com GRUB
2. **Entrada `_start`** → Ponto inicial após bootloader
3. **Buffer VGA** → Escreve em 0xb8000 (memória de vídeo)
4. **Formatação** → 2 bytes por caractere (char + cor)
5. **Loop infinito** → Mantém o kernel ativo

## ⚡ Características

- **Linguagem:** Rust (bare metal)
- **Target:** x86_64-unknown-none
- **Bootloader:** Multiboot compatível
- **Modo VGA:** Texto 80x25, 16 cores
- **Zero dependências:** Funciona offline

## ✨ Vantagens

- **Educativo:** Código simples para aprender OS development
- **Rápido:** Compilação sem dependências externas
- **Compatível:** Funciona em QEMU, VirtualBox e hardware real
- **Automatizado:** Script resolve problemas comuns

## ⚠️ Limitações

- Apenas exibe mensagens estáticas
- Sem sistema de interrupções
- Sem gerenciamento de memória dinâmica
- Sem drivers de hardware

---

**🎉 Pronto para começar?** Execute `./build-and-run.sh` e veja seu kernel funcionando!