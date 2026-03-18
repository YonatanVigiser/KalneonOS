build_mode  := "debug"
target_arch := arch()

default: (run "bios" target_arch)

build arch=target_arch:
    cargo build {{ if build_mode == "release" {"--release"} else {""} }} \
        --target targets/{{arch}}-kalneon_os.json

iso firmware="bios" arch=target_arch: (build arch)
    mkdir -p build/iso-{{arch}}/boot/grub
    cp build/{{arch}}-kalneon_os/{{build_mode}}/kernel build/iso-{{arch}}/boot/kernel
    cp grub.cfg build/iso-{{arch}}/boot/grub/grub.cfg
    grub-mkrescue -o build/kalneon_os-{{arch}}.iso build/iso-{{arch}} \
        -d /usr/lib/grub/{{ if firmware == "uefi" { "x86_64-efi" } else { "i386-pc" } }}

run firmware="bios" arch=target_arch: (iso firmware arch)
    {{ if arch == "x86" { "qemu-system-x86" } else { "qemu-system-x86_64" } }} \
        {{ if firmware == "uefi" { "-bios /usr/share/ovmf/OVMF.fd" } else { "" } }} \
        -cdrom build/kalneon_os-{{arch}}.iso -m 1024M -gdb tcp::26000 -S &
    gdb build/iso-{{arch}}/boot/kernel
