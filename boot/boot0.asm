org 0x7c00
bits 16

; boot0:

; The zero-stage bootloader. It will be loaded by the BIOS at 0x7c00.
; It job is to write boot1, located at the rest of the first cylinder, into memory, at 0x500 - 0x2500
; and then jump there. It will hang if something goes wrong.

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

  ; To ensure that the error occurred with int 0x13 - (and not print, for example), clear the carry flag 
  clc

  ; Load the second stage:
  mov ah, 2
  mov al, 17 ; Read the 2-18 sectors
  mov ch, 0  ; Cylinder 0
  mov cl, 2  ; Start from sector 2
  mov dh, 0  ; Head number 0
  ; The buffer for the code (0x500 - 0x2700)
  mov bx, 0x500
  ; dl should already be the drive the computer booted from
  ; (The BIOS sets it to be)
  int 0x13 ; Call the intterupt
  
  ; If there was an error
  jc .hlt

  jmp 0x500 ; Jumps to the first-stage bootloader

; Halt the machine
.hlt:
  hlt
  jmp .hlt

times 510 - ($-$$) db 0
dw 0xAA55
