build_mode  := "debug"
target_arch := "x86_64"
loader      := "limine"
limine_ver  := "v11.x-binary"

limine_bin := "build/limine/limine"
limine_dir := "build/limine"

iso_path := "build/kalneon_os-" + target_arch + "-" + loader + ".iso"

default: (run "bios" target_arch "false")

_limine:
    @test -x "{{limine_bin}}" && test -e "{{limine_dir}}/limine-bios.sys" \
      || (rm -rf build/limine \
          && git clone https://github.com/limine-bootloader/limine.git \
             --branch={{limine_ver}} --depth=1 build/limine \
          && make -C build/limine)

build arch=target_arch:
    cp targets/{{arch}}-kalneon_os.json targets/current-kalneon_os.json
    cargo build {{ if build_mode == "release" {"--release"} else {""} }}

iso arch=target_arch: (build arch)
    @just _iso-{{loader}} {{arch}}

_iso-limine arch=target_arch: _limine
    rm -rf build/iso-{{arch}}-limine
    mkdir -p build/iso-{{arch}}-limine/boot/limine build/iso-{{arch}}-limine/EFI/BOOT
    cp build/current-kalneon_os/{{build_mode}}/kernel build/iso-{{arch}}-limine/boot/kernel
    cp limine.conf build/iso-{{arch}}-limine/boot/limine/
    cp {{limine_dir}}/limine-bios.sys {{limine_dir}}/limine-bios-cd.bin \
       {{limine_dir}}/limine-uefi-cd.bin build/iso-{{arch}}-limine/boot/limine/
    cp {{limine_dir}}/{{ if arch == "x86" { "BOOTIA32.EFI" } else { "BOOTX64.EFI" } }} \
       build/iso-{{arch}}-limine/EFI/BOOT/
    xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
        -apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        build/iso-{{arch}}-limine -o build/kalneon_os-{{arch}}-limine.iso
    {{limine_bin}} bios-install build/kalneon_os-{{arch}}-limine.iso

_iso-grub arch=target_arch:
    rm -rf build/iso-{{arch}}-grub
    mkdir -p build/iso-{{arch}}-grub/boot/grub
    cp build/current-kalneon_os/{{build_mode}}/kernel build/iso-{{arch}}-grub/boot/kernel
    cp grub.cfg build/iso-{{arch}}-grub/boot/grub/
    grub-mkrescue -o build/kalneon_os-{{arch}}-grub.iso build/iso-{{arch}}-grub

run firmware="bios" arch=target_arch vnc="false": (iso arch)
    mkdir -p logs/
    {{ if arch == "x86" { "qemu-system-i386" } else { "qemu-system-x86_64" } }} \
        {{ if firmware == "uefi" { "-bios /usr/share/ovmf/OVMF.fd" } else { "" } }} \
        {{ if path_exists("/dev/kvm") == "true" { "-enable-kvm -cpu host" } else { "" } }} \
        -no-reboot -no-shutdown -d int,cpu_reset,guest_errors -D logs/qemu.log \
        -drive file=build/kalneon_os-{{arch}}-{{loader}}.iso,format=raw,if=ide,media=disk \
        -m 1024M -smp 4 \
        -serial file:logs/serial.log \
        -device VGA,xres=1280,yres=800,vgamem_mb=32 \
        -gdb tcp::26000 -S \
        {{ if vnc == "true" { "-vnc :1"} else { "" } }} &
    rust-gdb build/iso-{{arch}}-{{loader}}/boot/kernel
