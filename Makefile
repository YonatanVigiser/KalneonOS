BUILD_DIR := build
BOOT_DIR := boot
KERNEL_DIR := kernel
RUST_MODE ?= debug

KERNEL_X86_BIN := $(BUILD_DIR)/$(KERNEL_DIR)/legacy/kernel.bin
KERNEL_X86 := $(BUILD_DIR)/$(KERNEL_DIR)/i386-kalneon_os/$(if $(filter release,$(RUST_MODE)),release,debug)/kernel

MBR_BIN := $(BUILD_DIR)/legacy/mbr.bin
BOOT0_BIN := $(BUILD_DIR)/legacy/boot0.bin
BOOT1_BIN := $(BUILD_DIR)/legacy/boot1.bin
DISK_SIZE := 3000

IMAGE_X86_LEGACY := $(BUILD_DIR)/kalneonos-x86-legacy.img
ISO_X86 := $(BUILD_DIR)/kalneonos-x86.iso

GRUB_CFG_FILE := $(BOOT_DIR)/grub.cfg

.PHONY: all x86-legacy x86 clean $(KERNEL_X86_BIN) $(KERNEL_X86)

all: x86 x86-legacy

x86-legacy: $(IMAGE_X86_LEGACY)

$(IMAGE_X86_LEGACY): $(KERNEL_X86_BIN) $(MBR_BIN) $(BOOT0_BIN) $(BOOT1_BIN)
	dd if=/dev/zero of=$@ bs=512 count=$(DISK_SIZE)
	echo "1,,83,*" | sfdisk $@
	dd conv=notrunc if=$(MBR_BIN)   of=$@ bs=446 count=1 seek=0
	dd conv=notrunc if=$(BOOT0_BIN) of=$@ bs=512 count=1 seek=1
	dd conv=notrunc if=$(BOOT1_BIN) of=$@ bs=512 count=17 seek=2
	dd conv=notrunc if=$(KERNEL_X86_BIN) of=$@ bs=512 count=2048 seek=19

$(MBR_BIN) $(BOOT0_BIN) $(BOOT1_BIN):
	$(MAKE) -C $(BOOT_DIR)/legacy

$(KERNEL_X86_BIN):
	@mkdir -p $(dir $@)
	cd $(KERNEL_DIR) && cargo-objcopy $(if $(filter release,$(RUST_MODE)),--release,) --target targets/i386-kalneon_os.json --features legacy -- -O binary ../$@

x86: $(ISO_X86)

$(ISO_X86): $(KERNEL_X86)
	@mkdir -p $(BUILD_DIR)/isodir-x86/boot/grub
	cp $< $(BUILD_DIR)/isodir-x86/boot/kernel
	cp $(GRUB_CFG_FILE) $(BUILD_DIR)/isodir-x86/boot/grub/grub.cfg
	grub-mkrescue -o $@ $(BUILD_DIR)/isodir-x86 -d /usr/lib/grub/i386-pc

$(KERNEL_X86):
	cd $(KERNEL_DIR) && cargo build $(if $(filter release,$(RUST_MODE)),--release,) --target targets/i386-kalneon_os.json

clean:
	cd $(KERNEL_DIR) && cargo clean
	rm -rf $(BUILD_DIR)

