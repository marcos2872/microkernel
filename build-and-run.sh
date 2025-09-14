#!/bin/bash
# build-and-run.sh
# Script automatizado para compilar e executar o microkernel

set -e  # Para o script em caso de erro

echo "🦀 Compilando microkernel em Rust..."
cargo build --release

# Verifica se o kernel é válido para Multiboot
if ! grub-file --is-x86-multiboot target/x86_64-unknown-none/release/microkernel; then
    echo "❌ ERRO: Kernel não é reconhecido como Multiboot válido!"
    exit 1
fi

echo "✅ Kernel Multiboot válido"

# Cria e configura a imagem de disco (se não existir)
if [ ! -f disk.img ]; then
    echo "💾 Criando imagem de disco..."
    qemu-img create -f raw disk.img 50M

    echo "🔧 Configurando partição..."
    sudo losetup -P /dev/loop0 disk.img
    echo -e "n\np\n1\n\n\na\nw\n" | sudo fdisk /dev/loop0 >/dev/null 2>&1
    sudo partprobe /dev/loop0
    sudo mkfs.ext2 /dev/loop0p1 >/dev/null 2>&1

    echo "📁 Criando estrutura de boot..."
    sudo mkdir -p /mnt/bootdisk
    sudo mount /dev/loop0p1 /mnt/bootdisk
    sudo mkdir -p /mnt/bootdisk/boot/grub

    echo "⚙️ Configurando GRUB..."
    echo -e "set timeout=1\nset default=0\n\nmenuentry \"Microkernel Rust\" {\n    multiboot /boot/microkernel\n    boot\n}" | sudo tee /mnt/bootdisk/boot/grub/grub.cfg >/dev/null

    sudo grub-install --target=i386-pc --boot-directory=/mnt/bootdisk/boot /dev/loop0 >/dev/null 2>&1
    sudo umount /mnt/bootdisk
    sudo losetup -d /dev/loop0

    echo "✅ Imagem de disco criada com sucesso!"
fi

# Atualiza o kernel na imagem existente
echo "🔄 Atualizando kernel na imagem..."
sudo losetup -P /dev/loop0 disk.img
sudo mount /dev/loop0p1 /mnt/bootdisk
sudo cp target/x86_64-unknown-none/release/microkernel /mnt/bootdisk/boot/
sudo umount /mnt/bootdisk
sudo losetup -d /dev/loop0

# Executa no QEMU
echo "🚀 Iniciando microkernel no QEMU..."
echo "   Você verá:"
echo "   📺 *** MICROKERNEL RUST FUNCIONANDO! *** (branco)"
echo "   📺 Pressione Ctrl+Alt+G para sair do QEMU (amarelo)"
echo ""
echo "Para sair: Ctrl+Alt+G ou feche a janela"
echo ""

qemu-system-x86_64 -drive file=disk.img,format=raw -nic none
