[BITS 16]
[ORG 0x600]

start:
  cli
  ; Load data segments
  xor ax, ax
  mov ds, ax
  mov es, ax
  mov gs, ax
  mov fs, ax

  ; Load a stack (At address 0x7c00)
  mov ss, ax
  mov ax, 0x7c00
  mov sp, ax
  
  ; Relocate code to 0x600
  mov cx, 0x200
  lea si, 0x7c00
  lea di, 0x600
.copy_loop:
  mov al, byte [si]
  ; Set si to point to the active partition table entry
  mov byte [di], al
  inc si
  inc di
  loop .copy_loop

  ; Far jump to the relocated code (and enforce cs) 
  jmp 0x0:after_reloc

after_reloc:
  mov cx, 0x04
  mov bx, PTE1 - 0x10
.detect_active_partition:
  add bx, 0x10
  ; Test if partition is unused
  mov al, byte [bx + 0x04]
  cmp al, 0x0
  je .detect_active_partition
  ; Test if partition is active
  mov al, byte [bx]
  cmp al, 0x80
  je .load_active_partition
  loop .detect_active_partition

; If no active partition found, print error message
.not_found:
  lea si, active_partition_not_found_message
  call print_error
  jmp .error

.load_active_partition:
  ; Set si to point to the active partition table entry
  mov ax, 0x04
  sub ax, cx
  shl ax, 4
  add ax, PTE1
  mov si, ax

  push si ; Save si

  ; Set the dpa LBA values using values form the partition table entry
  add si, 0x08
  lea di, dap + 0x08
  mov cx, 0x04
.copy_lba_to_dap:
  mov al, byte [si]
  mov byte [di], al
  inc si
  inc di
  loop .copy_lba_to_dap

  ; Load the partition VBR to memory
  mov ah, 0x42
  lea si, dap
  int 0x13

  ; Test for error
  jnc .test_vbr

  lea si, vbr_copy_error_message
  call print_error
  jmp .error

.test_vbr:
  ; Test if VBR contains a boot signature
  cmp word [0x7DFE], 0xAA55
  je .jump_to_vbr

  lea si, active_partition_vbr_not_bootable_messgae
  call print_error
  jmp .error

.jump_to_vbr:
  pop si ; Restore si

  ; Jump to 0x7c00
  jmp 0x7c00

.error:
  hlt
  jmp .error

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

; Error Messages:
active_partition_not_found_message: db "MBR: Active partition was not found on disk! Machine halted!", 0x0
active_partition_vbr_not_bootable_messgae: db "MBR: Active partition's VBR is not bootable! Machine halted!", 0x0
vbr_copy_error_message: db "MBR: An error occurred while coping VBR from the disk! Machine halted!", 0x0

times 424 - ($-$$) db 0

dap:
  db 0x10   ; Size of DAP (16 bytes)
  db 0x00   ; Reserved
  dw 0x01   ; Read 1 sector
  dw 0x7c00 ; Buffer offset
  dw 0x0000 ; Buffer segment
  dq 0x0    ; LBA address (fill dynamically)

UDID dd 0x0 ; Unique Disk ID
dw 0x0 ; Reserved

; Partition Table:
PTE1 times 16 db 0x0
PTE2 times 16 db 0x0
PTE3 times 16 db 0x0
PTE4 times 16 db 0x0

dw 0xAA55
