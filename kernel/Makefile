BUILD_DIR=../build/kernel
TARGET_ARCH=i686
BINUTILS=~/opt/cross/bin/$(TARGET_ARCH)
LINKER=linker.ld
ELF=$(BUILD_DIR)/kalneonos.elf
BIN=$(BUILD_DIR)/kalneonos.bin

KERNEL_SRC := $(wildcard *.c)
KERNEL_OBJ := $(patsubst %.c, $(BUILD_DIR)/%.o, $(KERNEL_SRC))

all: $(BIN)

$(BUILD_DIR)/%.o: %.c
	$(BINUTILS)-elf-gcc -c $< -o $@ -std=gnu99 -ffreestanding -O2 -Wall -Wextra
  
$(ELF): $(KERNEL_OBJ)
	$(BINUTILS)-elf-ld -T $(LINKER) -o $(ELF) $(KERNEL_OBJ)

$(BIN): $(ELF)
	$(BINUTILS)-elf-objcopy -O binary $(ELF) $(BIN)

clean:
	rm $(BUILD_DIR)/*
