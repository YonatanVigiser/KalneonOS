build_mode := "debug"
target_arch := arch()
grub_arch := "i386-pc"

default: create_iso

build:
  cargo build {{ if build_mode == "release" {"--release"} else {""} }} --target targets/{{target_arch}}-kalneon_os.json

create_iso: build
  mkdir -p build/iso-{{target_arch}}/boot/grub
  cp build/{{target_arch}}-kalneon_os/{{build_mode}}/kernel build/iso-{{target_arch}}/boot/kernel
  cp grub.cfg build/iso-{{target_arch}}/boot/grub/grub.cfg
  grub-mkrescue -o build/kalneon_os-{{target_arch}}.iso build/iso-{{target_arch}} -d /usr/lib/grub/{{grub_arch}}
