[BITS 16]
[ORG 0x7c00]

; The zero-stage bootloader. It will be loaded by the BIOS\MBR at 0x7c00.
; It copys the first-stage bootloader from the disk to memory at 0x1000 - 0x3200,
; and then jump there. It will print the error and hang if something goes wrong.
; If loaded by MBR (meannig the boot drive is a hard drive), partition table entry is
; assumed to be located at ds:si

; Note: It will also disable intterupts, setup the segment selectors and the stack (at 0x7c00)
; for, so boot1 doens't need to do that.

boot:
  cli ; Disable intterupts  
  
  ; Setup a stack for the bootloader (top at 0x7c00)
  xor ax, ax
  mov ss, ax
  mov sp, 0x7c00

  ; Setup segment selectors, except ds
  mov es, ax
  mov fs, ax
  mov gs, ax

  ; If si is 0, probably using a VM without a proper disk, so don't treat it as a hard drive
  test si, si
  jz not_hard_dirve

  ; Test if the device is a hard drive or not 
  test dl, 0x80
  jnz hard_drive

not_hard_dirve:
  xor ax, ax
  mov ds, ax

  ; Load the next stage:
  mov ah, 2
  mov al, 17 ; Read the 2-18 sectors
  mov ch, 0  ; Cylinder 0
  mov cl, 2  ; Start from sector 2
  mov dh, 0  ; Head number 0
  ; The buffer for the code (0x1000 - 0x3200)
  mov bx, 0x1000
  int 0x13 ; Call the intterupt

  ; Test for errors
  jc error

  ; Far jump to the next stage bootloader
  jmp 0x0:0x1000


; Use LBA using the values from the partition rable entry at ds:si
hard_drive:
  push si ; Save si

  ; Set the dpa LBA values using values form the partition table entry
  add si, 0x08
  lea di, dap + 0x08
  mov cx, 0x04
.copy_lba_to_dap:
  mov al, byte [ds:si]
  mov byte [di], al
  inc si
  inc di
  loop .copy_lba_to_dap

  ; Load the partition VBR to memory
  mov ah, 0x42
  lea si, dap
  int 0x13
  
  ; Test for error
  jc error

  pop si ; Restore si

  ; Far jump to the next stage bootloader
  jmp 0x0:0x1000

; Print error message and halts the machine
error:
  lea si, memcopy_error_message
  call print_error
.hlt:
  hlt
  jmp .hlt

; Set a video mode, clear the screen, and write error
; Params:
;   Pointer to string at ds:si
print_error:
  ; Set the video mode to 0x03 (also clears the screen)
  mov ah, 0x00
  mov al, 0x03
  int 0x10

  ; Print only one char at a time
  mov cx, 0x01
  
  ; Page num is 0
  mov bh, 0x00

  ; Set the rows and columns
  mov dh, 0x00
  mov dl, 0x00
  
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
  mov ah, 0x0A
  int 0x10

  ; Loop
  inc dl
  jmp .print_loop

.ret:
  ret

; Messages:
memcopy_error_message: db "Unable to copy first-stage bootloader from memory. Machine halted!", 0x0

align 16
dap:
  db 0x10   ; Size of DAP (16 bytes)
  db 0x00   ; Reserved
  dw 0x11   ; Read 17 sectors
  dw 0x1000 ; Buffer offset
  dw 0x0000 ; Buffer segment
  dq 0x0    ; LBA address (fill dynamically)

times 510 - ($-$$) db 0
dw 0xAA55
