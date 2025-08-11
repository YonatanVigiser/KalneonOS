BUILD_DIR=../build/kernel

all:
	cargo build
	cargo objcopy -- -O binary --gap-fill=0x00 $(BUILD_DIR)/kernel.bin

