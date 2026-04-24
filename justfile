build_mode  := "debug"
target_arch := arch()

default: (run "bios" target_arch)

build arch=target_arch:
    cp targets/{{arch}}-kalneon_os.json targets/current-kalneon_os.json
    cargo build {{ if build_mode == "release" {"--release"} else {""} }}

iso firmware="bios" arch=target_arch: (build arch)
    mkdir -p build/iso-{{arch}}/boot/grub
    cp build/current-kalneon_os/{{build_mode}}/kernel build/iso-{{arch}}/boot/kernel
    cp grub.cfg build/iso-{{arch}}/boot/grub/grub.cfg
    grub-mkrescue -o build/kalneon_os-{{arch}}.iso build/iso-{{arch}} \
        -d /usr/lib/grub/{{ if firmware == "uefi" { "x86_64-efi" } else { "i386-pc" } }}

run firmware="bios" arch=target_arch: (iso firmware arch)
    {{ if arch == "x86" { "qemu-system-x86" } else { "qemu-system-x86_64" } }} \
        {{ if firmware == "uefi" { "-bios /usr/share/ovmf/OVMF.fd" } else { "" } }} \
        -cdrom build/kalneon_os-{{arch}}.iso -m 1024M -smp 4 -gdb tcp::26000 -S -d int,cpu_reset -D /tmp/qemu.log -enable-kvm -cpu host &
    gdb build/iso-{{arch}}/boot/kernel
