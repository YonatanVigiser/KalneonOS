BUILD_DIR=build
BOOT_DIR=boot
KERNEL_DIR=boot
BOOT_0=$(BUILD_DIR)/$(BOOT_DIR)/boot0.o
BOOT_1=$(BUILD_DIR)/$(BOOT_DIR)/boot1.o
KERNEL=$(BUILD_DIR)/$(KERNEL_DIR)/kernel.o
IMAGE_NAME=$(BUILD_DIR)/disk.img

all: clean image

.PHONY: boot image boot clean

build_dir:
	mkdir $(BUILD_DIR)
	mkdir $(BUILD_DIR)/$(BOOT_DIR)

boot: build_dir
	make -C $(BOOT_DIR)

image: boot
	dd if=/dev/zero of=$(IMAGE_NAME) bs=512 count=2097152
	dd conv=notrunc if=$(BOOT_0) of=$(IMAGE_NAME) bs=512 count=1 seek=0
	dd conv=notrunc if=$(BOOT_1) of=$(IMAGE_NAME) bs=512 count=17 seek=1
	dd conv=notrunc if=$(KERNEL) of=$(IMAGE_NAME) bs=512 count=2048 seek=18

clean:
	rm -rf $(BUILD_DIR)
