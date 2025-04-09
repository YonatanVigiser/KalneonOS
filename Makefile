BUILD_DIR=build
BOOT_DIR=boot
KERNEL_DIR=boot
BOOT_0=$(BUILD_DIR)/$(BOOT_DIR)/boot0.o
BOOT_1=$(BUILD_DIR)/$(BOOT_DIR)/boot1.o
KERNEL=$(BUILD_DIR)/$(KERNEL_DIR)/kernel.o
FLOPPY_DISK=$(BUILD_DIR)/disk.img

all: clean floppy

.PHONY: boot floppy boot clean

build_dir:
	mkdir $(BUILD_DIR)
	mkdir $(BUILD_DIR)/$(BOOT_DIR)

boot: build_dir
	make -C $(BOOT_DIR)

floppy: boot
	dd if=/dev/zero of=$(FLOPPY_DISK) bs=512 count=2880
	dd conv=notrunc if=$(BOOT_0) of=$(FLOPPY_DISK) bs=512 count=1 seek=0
	dd conv=notrunc if=$(BOOT_1) of=$(FLOPPY_DISK) bs=512 count=17 seek=1
	dd conv=notrunc if=$(KERNEL) of=$(FLOPPY_DISK) bs=512 count=2048 seek=2

clean:
	rm -rf $(BUILD_DIR)
