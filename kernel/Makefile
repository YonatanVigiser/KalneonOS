BUILD_DIR=../build/kernel

KERNEL_SRC := $(wildcard *.c)
KERNEL_OBJ := $(patsubst %.c, $(BUILD_DIR)/%.o, $(KERNEL_SRC))

all: $(KERNEL_OBJ)

$(BUILD_DIR)/%.o: %.c
	~/opt/cross/bin/i686-elf-gcc -c $< -o $@ -std=gnu99 -ffreestanding -O2 -Wall -Wextra

clean:
	rm $(BUILD_DIR)/*
