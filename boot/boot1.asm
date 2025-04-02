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

  ; Set background color
  mov ah, 0x0B
  mov bh, 0x00
  mov bl, 0x40 ; Background color red, border color black
  int 0x10

  ; Print booting message:
  lea si, boot_message
  mov ah, 0x00 ; row 0
  mov al, 0x0F ; white
  call print_line

  jmp .hlt

  ; Enable A20 line
  call enable_a20
  jc .a20_error

  ; Enter unreal mode
  call enter_unreal_mode

  ; Load the kernel
  call load_kernel
  jc .memcpy_error

  jmp .hlt

.a20_error:
  lea si, a20_error_message
  mov ah, 0x01 ; row 1
  mov al, 0x40 ; red back, white text
  call print_line
  jmp .hlt

.memcpy_error:
  lea si, memcpy_error_message
  mov ah, 0x01 ; row 1
  mov al, 0x40 ; red back, white text
  call print_line
  jmp .hlt

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

; Enters unreal mode. Will enter 32-bit protected mode and then return
; to real mode without reloading the segment selectors,
; which causes the proccessor to enter "unreal mode"
enter_unreal_mode:
  ; Save ds real mode value
  push ds

  ; Enter 32-bit protected mode:

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

  jmp 0x8:.unreal_mode ; reload cs

.unreal_mode:
  ; Returns to unreal mode
  mov eax, cr0
  and eax, 0xFFFFFFFE
  mov cr0, eax
  
  jmp 0x0:.ret ; reload cs  

.ret:
  pop ds
  ret

; This will load the kernel from disk to memory (100000-1FFFFF).
; It should be called only in unreal mode.
; Will return carry if failed, else carry clear
load_kernel:
  clc ; clear carry
  
  mov cx, 0x20 ; Read 64 segments and copy 32 times (64*32*0.5=1024K)
  mov ah, 0x42 ; Function code
  lea si, dpa  ; load DPA
  mov edi, 0x100000
  mov es, 0x16 ; Data segment offset
.copy_loop:
  int 0x13
  jc .ret ; If error, return

  mov bx, 0x0
.copy_from_buffer_loop:
  ; Copy the memory from buffer to new location:
  mov dx, [0x7000:0x8000+bx]
  mov [es:edi], dx

  ; Loop:
  add edi, 1
  add bx, 1
  test bx, 0xFF ; 256*2=512 bytes
  jne .copy_from_buffer_loop

  ; Add the reading starting address 
  ; 0x8000 (64*512=32,768):

  mov ebx, [si+8]
  add ebx, 0x8000
  mov [si+8], ebx

  loop .copy_loop

.ret 
  ret
  

; 32-bit protected mode code
[BITS 32]

main32:

; Data:

; Text:
boot_message: db "First-stage booting, please wait...", 0x0
a20_error_message: db "CRITICAL ERROR: Unable to enable A20 line. Machine halted!", 0x0
memcpy_error_message: db "CRITICAL ERROR: Unable to copy the kernel form disk. Machine halted!", 0x0

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

; DPA - (disk address packet)
dpa:
  db 0x10      ; Size of DPA (16-bytes)
  db 0x0       ; Reserved
  dw 0x40      ; Read 64 sectors
  dw 0x8000    ; Buffer offset
  dw 0x7000    ; Buffer segment
  dd 0x2400    ; Starting read address LBA lower 32-bits
  dd 0x0       ; Starting read address LBA upper 16-bits (16-bits unused)

