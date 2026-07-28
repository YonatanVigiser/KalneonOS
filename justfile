build_mode  := "debug"
target_arch := "x86_64"
limine_ver  := "v12.x-binary"

limine_bin := "build/limine/limine"
limine_dir := "build/limine"

default: (run "bios" target_arch "true")

_limine:
    @test -x "{{limine_bin}}" && test -e "{{limine_dir}}/limine-bios.sys" \
      || (rm -rf build/limine \
          && git clone https://github.com/limine-bootloader/limine.git \
             --branch={{limine_ver}} --depth=1 build/limine \
          && make -C build/limine)

build arch=target_arch:
    cp targets/{{arch}}-kalneon_os.json targets/current-kalneon_os.json
    cargo build {{ if build_mode == "release" {"--release"} else {""} }}

iso arch=target_arch: (build arch) _limine
    mkdir -p build/iso-{{arch}}/boot/limine build/iso-{{arch}}/EFI/BOOT
    cp build/current-kalneon_os/{{build_mode}}/kernel build/iso-{{arch}}/boot/kernel
    cp limine.conf build/iso-{{arch}}/boot/limine/
    cp {{limine_dir}}/limine-bios.sys {{limine_dir}}/limine-bios-cd.bin \
       {{limine_dir}}/limine-uefi-cd.bin build/iso-{{arch}}/boot/limine/
    cp {{limine_dir}}/{{ if arch == "x86" { "BOOTIA32.EFI" } else { "BOOTX64.EFI" } }} \
       build/iso-{{arch}}/EFI/BOOT/
    xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
        -apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        build/iso-{{arch}} -o build/kalneon_os-{{arch}}.iso
    {{limine_bin}} bios-install build/kalneon_os-{{arch}}.iso

run firmware="bios" arch=target_arch vnc="false": (iso arch)
    {{ if arch == "x86" { "qemu-system-i386" } else { "qemu-system-x86_64" } }} \
        {{ if firmware == "uefi" { "-bios /usr/share/ovmf/OVMF.fd" } else { "" } }} \
        {{ if path_exists("/dev/kvm") == "true" { "-enable-kvm -cpu host" } else { "" } }} \
        -drive file=build/kalneon_os-{{arch}}.iso,format=raw,if=ide,media=disk \
        -m 1024M -smp 4 \
        -gdb tcp::26000 -S -d cpu_reset -D /tmp/qemu.log \
        {{ if vnc == "true" { "-vnc :1"} else { "" } }} &
    gdb-multiarch build/iso-{{arch}}/boot/kernel
