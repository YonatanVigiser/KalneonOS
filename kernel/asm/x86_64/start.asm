extern main

section .text._start
global _start
_start:
  ; Disable interrupts:
  cli

  ; Load a simple GDT:
  lgdt [gdt_desc]

  jmp 0x08::.reload_seg

.reload_seg:
  ; Reload the segments
  mov ax, 0x10
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  mov ss, ax

  ; Load a temp "valid" stack for init:
  mov rsp, stack_top

  mov rdi, rax
  mov rsi, rbx

  ; Jump to kernel main
  call main

  ; If kernel returns:
  cli
.loop:
  hlt
  jmp .loop

; GDT for 64-bit mode
gdt:

gdt_null:
  dq 0x0

; 64-bit code segment
gdt_code:
  dw 0xFFFF    ; Limit (0-15)
  dw 0x0000    ; Base (0-15)
  db 0x00      ; Base (16-23)
  db 10011010b ; Access byte (Present, Ring 0, Code, Execute/Read)
  db 10101111b ; Limit (16-19) + Flags (Granularity, Long mode)
  db 0x00      ; Base (24-31)

; 64-bit data segment
gdt_data:
  dw 0xFFFF    ; Limit (0-15)
  dw 0x0000    ; Base (0-15)
  db 0x00      ; Base (16-23)
  db 10010010b ; Access byte (Present, Ring 0, Data, Read/Write)
  db 11001111b ; Limit (16-19) + Flags
  db 0x00      ; Base (24-31)

gdt_end:

gdt_desc:
  dw gdt_end - gdt - 1
  dq gdt

section .bss.stack
align 16
stack: resb 0x10000
stack_top:
