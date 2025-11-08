BUILD_DIR=../../build/legacy

BOOT_SRC := $(wildcard *.asm)
BOOT_OBJ := $(patsubst %.asm, $(BUILD_DIR)/%.bin, $(BOOT_SRC))

all: $(BOOT_OBJ)

$(BUILD_DIR)/%.bin: %.asm
	mkdir -p $(BUILD_DIR)
	nasm -f bin $< -o $@
