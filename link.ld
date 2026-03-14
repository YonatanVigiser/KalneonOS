ENTRY(_start)

SECTIONS {
  . = 0x00100000;

  .text : {
    . = ALIGN(8);
    KEEP(*(.multiboot))
    *(.text*)
  }

  .rodata : {
    *(.rodata*)
  }

  .data : {
    *(.data*)
  }

  .bss : {
    *(.bss*)
    *(COMMON)
    *(.stack*)
  }

  .heap (NOLOAD) : ALIGN(4K) {
    __heap_start = .;
    . += 1M;
    __heap_end = .;
  }
}
