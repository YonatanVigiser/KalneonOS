org 0x500
bits 16

main:
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

  hlt ; TEMP!

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

; Data:

; Text
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
  db 10011010b ; Accses byte

gdb_end

gdb_desc:
  dw gdb_end - gdb - 1
  dd gdb
