ENTRY(_start)

SECTIONS {
  . = 0x00100000;

  .multiboot : ALIGN(8) {
    *(.multiboot)
  }

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
    . = ALIGN(64);
    stack_bottom = .;
    . += 0x100000;
    stack_top = .;
  }

  .heap (NOLOAD) : ALIGN(4K) {
    __heap_start = .;
    . += 1M;
    __heap_end = .;
  }
  
  /DISCARD/ : {
    *(.eh_frame)
    *(.eh_frame_hdr)
    *(.note*)
  }
}
