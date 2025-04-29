BUILD_DIR=build
BOOT_DIR=boot
KERNEL_DIR=kernel
BOOT_0=$(BUILD_DIR)/$(BOOT_DIR)/boot0.o
BOOT_1=$(BUILD_DIR)/$(BOOT_DIR)/boot1.o
KERNEL=$(BUILD_DIR)/$(KERNEL_DIR)/kernel.o
CROSS_COMPILER=~/opt/cross/bin/i686-elf-gcc 
LINKER=linker.ld
OS_NAME=kalneonos
DISK_SEGMENTS=3000
IMAGE_NAME=$(BUILD_DIR)/disk.img

all: clean image

.PHONY:
build_dir:
	mkdir $(BUILD_DIR)
	mkdir $(BUILD_DIR)/$(BOOT_DIR)
	mkdir $(BUILD_DIR)/$(KERNEL_DIR)

.PHONY:
boot: build_dir
	make -C $(BOOT_DIR)

.PHONY:
kernel: build_dir
	make -C $(KERNEL_DIR)
	$(CROSS_COMPILER) -T $(LINKER) -o $(OS_NAME).bin -ffreestanding -O2 -nostdlib $(KERNEL)

.PHONY:
image: boot kernel
	dd if=/dev/zero of=$(IMAGE_NAME) bs=512 count=$(DISK_SEGMENTS)
	dd conv=notrunc if=$(BOOT_0) of=$(IMAGE_NAME) bs=512 count=1 seek=0
	dd conv=notrunc if=$(BOOT_1) of=$(IMAGE_NAME) bs=512 count=17 seek=1
	dd conv=notrunc if=$(OS_NAME).bin of=$(IMAGE_NAME) bs=512 count=2048 seek=18

.PHONY:
clean:
	rm -rf $(BUILD_DIR)
