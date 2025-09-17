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
    LOOP_DEVICE=$(sudo losetup -f -P --show disk.img)
    echo -e "n\np\n1\n\n\na\nw\n" | sudo fdisk $LOOP_DEVICE >/dev/null 2>&1
    sudo partprobe $LOOP_DEVICE
    sudo mkfs.ext2 ${LOOP_DEVICE}p1 >/dev/null 2>&1

    echo "📁 Criando estrutura de boot..."
    sudo mkdir -p /mnt/bootdisk
    sudo mount ${LOOP_DEVICE}p1 /mnt/bootdisk
    sudo mkdir -p /mnt/bootdisk/boot/grub

    echo "⚙️ Configurando GRUB..."
    echo -e "set timeout=1\nset default=0\n\nmenuentry \"Microkernel Rust\" {\n    multiboot /boot/microkernel\n    boot\n}" | sudo tee /mnt/bootdisk/boot/grub/grub.cfg >/dev/null

    sudo grub-install --target=i386-pc --boot-directory=/mnt/bootdisk/boot $LOOP_DEVICE >/dev/null 2>&1
    sudo umount /mnt/bootdisk
    sudo losetup -d $LOOP_DEVICE

    echo "✅ Imagem de disco criada com sucesso!"
fi

# Atualiza o kernel na imagem existente
echo "🔄 Atualizando kernel na imagem..."

# Libera dispositivos loop ocupados se necessário
EXISTING_LOOP=$(sudo losetup -j disk.img | cut -d: -f1)
if [ ! -z "$EXISTING_LOOP" ]; then
    echo "⚠️  Liberando dispositivo loop ocupado: $EXISTING_LOOP"
    sudo umount /mnt/bootdisk 2>/dev/null || true
    sudo losetup -d $EXISTING_LOOP 2>/dev/null || true
fi

# Cria ponto de montagem se não existir
sudo mkdir -p /mnt/bootdisk

# Monta e atualiza o kernel
LOOP_DEVICE=$(sudo losetup -f -P --show disk.img)
sudo mount ${LOOP_DEVICE}p1 /mnt/bootdisk
sudo cp target/x86_64-unknown-none/release/microkernel /mnt/bootdisk/boot/
sudo umount /mnt/bootdisk
sudo losetup -d $LOOP_DEVICE

# Executa no QEMU
echo "🚀 Iniciando microkernel no QEMU..."
echo "   Você verá:"
echo "   📺 *** MICROKERNEL RUST FUNCIONANDO! *** (branco)"
echo "   📺 Pressione Ctrl+Alt+G para sair do QEMU (amarelo)"
echo ""
echo "Para sair: Ctrl+Alt+G ou feche a janela"
echo ""

qemu-system-x86_64 -drive file=disk.img,format=raw -nic none
