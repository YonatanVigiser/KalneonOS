ENTRY(_start)

SECTIONS {
  . = 0x00100000;

  .multiboot : {
  }

  .text : {
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
