ENTRY(_start)

SECTIONS {
  . = 0x00100000;

  .text : {
    KEEP(*(.text._start))
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
  }

  .stack : ALIGN(16) {
    KEEP(*(.stack))
  }

  .heap (NOLOAD) : ALIGN(4K) {
    __heap_start = .;
    . += 1M;
    __heap_end = .;
  }
}
