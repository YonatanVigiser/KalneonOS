BUILD_DIR=build
BOOT_DIR=boot
BOOT_0=$(BUILD_DIR)/boot/boot0.o
BOOT_1=$(BUILD_DIR)/boot/boot1.o
FLOPPY_DISK_NAME=disk.img

all: floppy

.PHONY: floppy boot clean

boot:
	make -C $(BOOT_DIR)

floppy: boot
	dd if=/dev/zero of=$(FLOPPY_DISK_NAME) bs=512 count=2880
	dd conv=notrunc if=$(BOOT_0) of=$(FLOPPY_DISK_NAME) bs=512 count=1 seek=0
	dd conv=notrunc if=$(BOOT_1) of=$(FLOPPY_DISK_NAME) bs=512 count=17 seek=1

clean:
	make -C $(BOOT_DIR) clean
