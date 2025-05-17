BUILD_DIR=../build/boot

BOOT_SRC := $(wildcard *.asm)
BOOT_OBJ := $(patsubst %.asm, $(BUILD_DIR)/%.bin, $(BOOT_SRC))

all: $(BOOT_OBJ)

$(BUILD_DIR)/%.bin: %.asm
	nasm -f bin $< -o $@

clean:
	rm $(BUILD_DIR)/*
