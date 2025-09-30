extern main

global _start
_start:
  ; Load a temp "valid" stack for init:
  mov esp, stack_top

  ; Save the passed parameters to the kernel
  push eax ; Magic value from bootloader (indentifier)
  push ebx ; Boot Info pointer

  ; Load a simple GDT:
  lgdt [gdt_desc]

  ; Reload cs
  jmp 0x08:.reload_seg

.reload_seg:
  ; Reload the segments
  mov ax, 0x10
  mov ds, ax
  mov es, ax
  mov fs, ax 
  mov gs, ax

  ; Jump to kernel main
  call main

  ; If kernel returns:
  cli
.loop:
  hlt
  jmp .loop

gdt:

gdt_null:
  dq 0x0

; Base - 0x0
; Limit - 0xFFFFF
gdt_code:
  dw 0xFFFF    ; Limit (0-15)
  dw 0x0000    ; Base (0-15)
  db 0x00      ; Base (16-23)
  db 10011010b ; Accses byte
  db 11001111b ; Limit (16-19) + Flags
  db 0x00      ; Limit (24-31)

; Base - 0x0
; Limit - 0xFFFFF
gdt_data:
  dw 0xFFFF    ; Limit (0-15)
  dw 0x0000    ; Base (0-15)
  db 0x00      ; Base (16-23)
  db 10010010b ; Accses byte
  db 11001111b ; Limit (16-19) + Flags
  db 0x00      ; Limit (24-31)

gdt_end:

gdt_desc:
  dw gdt_end - gdt - 1
  dd gdt

section .stack
align 16
stack: resb 0x10000
stack_top:
