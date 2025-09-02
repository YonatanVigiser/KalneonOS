BUILD_DIR=build
BOOT_DIR=boot
KERNEL_DIR=kernel
MBR=$(BUILD_DIR)/$(BOOT_DIR)/mbr.bin
BOOT_0=$(BUILD_DIR)/$(BOOT_DIR)/boot0.bin
BOOT_1=$(BUILD_DIR)/$(BOOT_DIR)/boot1.bin
KERNEL=$(BUILD_DIR)/$(KERNEL_DIR)/kernel.bin
DISK_SIZE=3000
IMAGE_NAME=$(BUILD_DIR)/disk.img

all: clean image

.PHONY:
build_dir:
	mkdir -p $(BUILD_DIR)
	mkdir -p $(BUILD_DIR)/$(BOOT_DIR)
	mkdir -p $(BUILD_DIR)/$(KERNEL_DIR)

.PHONY:
boot: build_dir
	make -C $(BOOT_DIR)

.PHONY:
kernel: build_dir
	make -C $(KERNEL_DIR)

.PHONY:
image: boot kernel
	dd if=/dev/zero of=$(IMAGE_NAME) bs=512 count=$(DISK_SIZE)
	echo "1,,83,*" | sfdisk $(IMAGE_NAME)
	dd conv=notrunc if=$(MBR) 	 of=$(IMAGE_NAME) bs=446 count=1 seek=0
	dd conv=notrunc if=$(BOOT_0) of=$(IMAGE_NAME) bs=512 count=1 seek=1
	dd conv=notrunc if=$(BOOT_1) of=$(IMAGE_NAME) bs=512 count=17 seek=2
	dd conv=notrunc if=$(KERNEL) of=$(IMAGE_NAME) bs=512 count=2048 seek=19

.PHONY:
clean:
	rm -rf $(BUILD_DIR)/$(BOOT_DIR)

