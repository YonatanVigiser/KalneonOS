[BITS 16]
[ORG 0x500]

; The zero-stage bootloader will jump here.
; Will print another booting message, enable A20,
; and enter 32-bit protected mode:
main16:
  ; Set the page number to 1
  mov ah, 0x03
  mov bh, 0x01
  int 0x10

  ; Print booting message:
  lea si, .boot_message
  mov ah, 0x00 ; row 0
  mov al, 0x0F ; white
  call print_line

  ; Enable A20 line
  call enable_a20
  jc .a20_error

  ; Enter 32-bit protected mode: 
  call enter_p32_mode

  jmp .hlt

.a20_error:
  lea si, .a20_error_message
  mov ah, 0x01 ; row 1
  mov al, 0x0F ; white
  call print_line

.hlt:
  jmp .hlt

; Print a message to the screen
; Params:
;   Pointer to string at ds:si
;   ah - row
;   al - color
print_line:
  push ax
  push bx
  push cx
  push dx

  ; Print only one char at a time
  mov cx, 1
  
  ; Page num is 1
  mov bh, 1

  ; Set the text color
  mov bl, al
  
  ; Set the rows and columns
  mov dh, ah
  mov dl, 0
  
.print_loop:
  ; Move cursor
  mov ah, 0x02
  int 0x10
  
  ; Copy the char to al
  lodsb

  ; Check if null
  or al, al
  jz .ret

  ; Print the char
  mov ah, 0x09
  int 0x10

  ; Loop
  add dl, 1
  jmp .print_loop

.ret:
  pop dx
  pop cx
  pop bx
  pop ax
  ret

; Enables the A20 line
; Clear carry flag on success, set it on error
enable_a20:
  push ax
  clc ; Clears the carry flag
  
.ret:
  pop ax
  ret

; Enters 32-bit protected mode.
; will setup a new stack, reload segments
; and jumps to the 32-bit mode code
enter_32p_mode:
  ; Load a GDT
  lgdt [gdt_desc]

  ; Enable 32-bit protected mode in control register
  mov eax, cr0
  or eax, 1
  mov cr0, eax 

  ; Reload the segments
  xor ax, ax
  mov ds, ax
  mov es, ax
  mov fs, ax 
  mov gs, ax

  ; Setup protected mode stack
  mov sp, 0

  jmp 0x8:main32 ; Jumps to the 32-bit mode and reload cs

; This should be called once unreal mode have been entered.
; It will load the kernel from disk to memory, and return to protected mode.
load_kernel:

.ret:
  ; Return to 32-bit protected mode

  

; 32-bit protected mode code
[BITS 32]

main32:
  


  

; Data:

; Text:
.boot_message: db "First-stage booting, please wait...", 0x0
.a20_error_message: db "CRITICAL ERROR: Unable to enable A20 line. Machine halted!", 0x0

; GDB table (temporary - for entering protected mode):
gdb:

gdb_null:
  dq 0x0

; Base - 0x0
; Limit - 0xFFFFF
gdb_code:
  dw 0xFFFF    ; Limit (0-15)
  dw 0x0000    ; Base (0-15)
  db 0x00      ; Base (16-23)
  db 10011011b ; Accses byte
  db 11001111b ; Limit (16-19) + Flags
  db 0x00      ; Limit (24-31)

; Base - 0x0
; Limit - 0xFFFFF
gdb_data:
  dw 0xFFFF    ; Limit (0-15)
  dw 0x0000    ; Base (0-15)
  db 0x00      ; Base (16-23)
  db 10010011b ; Accses byte
  db 11001111b ; Limit (16-19) + Flags
  db 0x00      ; Limit (24-31)

gdb_end

gdb_desc:
  dw gdb_end - gdb - 1
  dd gdb

