ENTRY(_start)

SECTIONS {
  /* Kernel offset: */
  . = 1M;

  .text BLOCK(4K) : ALIGN(4K) {
    KEEP(*(.text._start))
    *(.text)
  }

  .rodata BLOCK(4K) : ALIGN(4K) {
    *(.rodata)
  }

  .data BLOCK(4K) : ALIGN(4K) {
    *(.data)
  }

  .bss BLOCK(4K) : ALIGN(4K) {
    *(COMMON)
    *(.bss)
  }
}
