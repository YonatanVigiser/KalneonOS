BUILD_DIR=build
BOOT_DIR=boot
BOOT_0=$(BUILD_DIR)/$(BOOT_DIR)/boot0.o
BOOT_1=$(BUILD_DIR)/$(BOOT_DIR)/boot1.o
FLOPPY_DISK=$(BUILD_DIR)/disk.img

all: floppy

.PHONY: boot floppy boot clean

build_dir:
	mkdir $(BUILD_DIR)
	mkdir $(BUILD_DIR)/$(BOOT_DIR)

boot: build_dir
	make -C $(BOOT_DIR)

floppy: clean boot
	dd if=/dev/zero of=$(FLOPPY_DISK) bs=512 count=2880
	dd conv=notrunc if=$(BOOT_0) of=$(FLOPPY_DISK) bs=512 count=1 seek=0
	dd conv=notrunc if=$(BOOT_1) of=$(FLOPPY_DISK) bs=512 count=17 seek=1

clean:
	rm -rf $(BUILD_DIR)/*
	rmdir $(BUILD_DIR)

