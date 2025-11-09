ENTRY(_start)

SECTIONS {
  . = 0x00200000;

  .text : {
    KEEP(*(.text._start))
    . = ALIGN(8)
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
  }

  .stack : ALIGN(16) {
    KEEP(*(.stack))
  }

  .heap (NOLOAD) : ALIGN(4K) {
    __heap_start = .;
    . += 4M;
    __heap_end = .;
  }
}
