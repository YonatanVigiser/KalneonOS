BUILD_DIR=../build/boot

BOOT_SRC := $(wildcard *.asm)
BOOT_OBJ := $(patsubst %.asm, $(BUILD_DIR)/%.o, $(BOOT_SRC))

all: $(BOOT_OBJ)

$(BUILD_DIR)/%.o: %.asm
	nasm -f bin $< -o $@

clean:
	rm $(BUILD_DIR)/*
