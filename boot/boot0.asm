org 0x7c00
bits 16

; boot0:

; The zero-stage bootloader. It will be loaded by the BIOS at 0x7c00.
; It job is to write boot1, located at the rest of the first cylinder, into memory, at 0x500 - 0x2500
; and then jump there. It will also print a booting message and alert the user if something
; went wrong.

; Note: It will also disable intterupts, setup the segment selectors and the stack (at 0x7c00)
; for, so boot1 doens't need to do that.

boot:
  cli ; Disable intterupts  
  
  ; Setup a stack for the bootloader (top at 0x7c00)
  xor ax, ax
  mov ss, ax
  mov sp, 0x7c00

  ; Setup all segment selectors
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax

  ; Print booting message
  lea si, .boot_message
  mov ah, 0x0C ; row 12
  mov al, 0x0F ; white color
  call print_line

  ; To ensure that the error occurred with int 0x13 - (and not print, for example), clear the carry flag 
  clc

  ; Load the second stage:
  mov ah, 2
  mov al, 17 ; Read the 2-18 sectors
  mov ch, 0 ; Cylinder 0
  mov cl, 2 ; Start from sector 2
  mov dh, 0 ; Head number 0
  mov bx, 0x500 ; The buffer for the code (0x500 - 0x2500)
  ; dl should already be the drive the computer booted from
  ; (They BIOS sets it to be)
  int 0x13 ; Call the intterupt
  
  ; If there was an error
  jc .error

  jmp 0x500 ; Jumps to the first-stage bootloader
  
  ; In case the first-stage bootloader returns
  jmp .hlt

.error:
  lea si, .boot_failed_message
  mov ah, 0x0D ; row 13
  mov al, 0x0F ; white color
  call print_line

; Halt the machine
.hlt:
  jmp .hlt

.boot_message: db "Zero-Stage Booting...", 0x0
.boot_failed_message: db "An error occurred while trying to copy the first-stage bootloader from disk, machine halted!", 0x0

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
  
  ; Page num is 0
  mov bh, 0

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

; Fill the rest of the 510 bytes with zeros and add the boot signature 0xAA55
times 510 - ($-$$) db 0
dw 0xAA55
