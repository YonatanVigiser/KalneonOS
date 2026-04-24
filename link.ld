ENTRY(_start)

SECTIONS {
  . = 0x00100000;
  __phys_start = .;

  .multiboot : ALIGN(8) {
    *(.multiboot)
  }

  .boot.text : {
    KEEP(*(.text._start))
  }

  .boot.rodata : {
    *(.boot.rodata)
  }

  .boot.bss (NOLOAD) : {
    *(.boot.bss)
  }

  .boot.stack (NOLOAD) : ALIGN(64) {
    __boot_stack_bottom = .;
    . += 4K;
    __boot_stack_top = .;
  }

  __high_sections_lma = .;

  __vma_start = 0xffffffff80000000;
  . += __vma_start;

  .text : AT(__high_sections_lma) ALIGN(4K) {  
    __text_start = .;
    *(.text*)
    __text_end = .;
  }

  .rodata : ALIGN(4K) {
    __rodata_start = .;
    *(.rodata*)
    __rodata_end = .;
  }

  .data : ALIGN(4K) {
    __data_start = .;
    *(.data*)
    __data_end = .;
  }

  .bss (NOLOAD) : ALIGN(4K) {
    __bss_start = .;
    *(.bss*)
    *(COMMON)
    __bss_end = .;
  }

  .bsp_stack (NOLOAD) : ALIGN(4K) {
    __bsp_stack_guard = .;
    . += 4K;
    __bsp_stack_buttom = .;
    . += 1M;
    __bsp_stack_top = .;
  }
  
  __vma_end = .;

  __phys_end = __vma_end - __vma_start; /* contains the low sections inside of it */

  /DISCARD/ : {
    *(.eh_frame)
    *(.eh_frame_hdr)
    *(.note*)
  }
}
