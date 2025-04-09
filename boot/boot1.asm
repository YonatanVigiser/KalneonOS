[BITS 16]
[ORG 0x500]

; The zero-stage bootloader will jump here.
; Will print another booting message, enable A20,
; and enter 32-bit protected mode:
main16:
  mov [boot_disk], dl ; Save the boot disk number

  call init_screen

  ; Enable A20 line
  call enable_a20
  jc .a20_error

  ; Print A20 enabled message
  lea si, a20_enabled_message
  mov al, 0x90 ; Black
  call print_line

  ; Enter unreal mode
  call enter_unreal_mode

  ; Load the kernel
  call load_kernel
  jc .memcpy_error

  ; Print kernel copy message
;  lea si, memcpy_success_message
;  mov al, 0x90 ; Black
;  call print_line
  jmp .hlt

.a20_error:
  lea si, a20_error_message
  mov al, 0x40 ; Red background, black text
  call print_line
  jmp .hlt

.memcpy_error:
  lea si, memcpy_error_message
  mov al, 0x40 ; Red background, black text
  call print_line
  jmp .hlt

.hlt:
  hlt ; For debugging
  jmp .hlt

; Set video mode, clear the screen, sets a background color (Magenta),
; and print boot message
init_screen:
  ; Set the video mode to 0x03
  mov ah, 0x00
  mov al, 0x03
  int 0x10

  ; Clear the screen and set background color
  mov ah, 0x06
  mov al, 0x00 ; Clear all
  mov bh, 0x90 ; Purple blue color
  mov ch, 0x00 ; y=0
  mov cl, 0x00 ; x=0
  mov dh, 0x19 ; y=25
  mov dl, 0x50 ; x=80
  int 0x10

  ; Print booting message:
  lea si, boot_message
  mov al, 0x90 ; Black text 
  call print_line

.ret:
  ret

; Print a message to the screen
; Params:
;   Pointer to string at ds:si
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
  mov dh, [screen_line]
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
  mov ah, [screen_line]
  add ah, 1
  mov [screen_line], ah
  pop dx
  pop cx
  pop bx
  pop ax
  ret

; Enables the A20 line
; Clear carry flag on success, set it on error
enable_a20:
  ; First, test if A20 is already enabled
  call test_a20
  cmp ax, 1
  je .ret

  mov bl, 0x03 ; If BIOS support check fails, we assume both are supported

  ; Check support via bios:
  mov ax, 0x2403
  int 0x15
  ; BIOS int 0x15 function isn't supported, so we will skip the call to the BIOS method:
  jc .keyboard_controller 
  mov bl, al

.bios:
  mov ax, 0x2401
  int 0x15
  jc .keyboard_controller  ; int 0x15 not supported
  test ah, ah
  jne .keyboard_controller ; int 0x15 not supported
  call test_a20
  test ax, ax
  jnz .ret

  ; test if keyboard controller is supported 
  test bl, 1
  jnz .keyboard_controller

  ; test if fast gate is supported
  test bl, 2
  jnz .fast_gate

.keyboard_controller:
  ; Disable keyboard
  call keyboard_wait_bit_1
  jc .controller_end
  mov al, 0xAD
  out 0x64, al

  ; Read from input
  call keyboard_wait_bit_1
  jc .controller_end
  mov al, 0xD0
  out 0x64, al

  call keyboard_wait_bit_0
  jc .controller_end
  in al, 0x60
  push ax

  ; Write to output
  call keyboard_wait_bit_1
  jc .controller_end
  mov al, 0xD1   
  out 0x64, al

  call keyboard_wait_bit_1
  jc .controller_end
  pop ax
  or al, 2
  out 0x60, al
  
  ; Re-enable keyboard controller
  call keyboard_wait_bit_1
  jc .controller_end
  mov al, 0xAE
  out 0x64, al

  ; Wait for controller
  call keyboard_wait_bit_1
  jc .controller_end

  ; Check if the a20 line was enabled
  call test_a20
  test al, al
  jnz .ret

.controller_end:
  ; Test if the fast gate is supported
  test bl, 2
  jnz .fast_gate

  jmp .fail  

.fast_gate:
  ; Test if we need to use the fast gate method
  in al, 0x92
  test al, 0x02
  jnz .ret

  ; Enable using the fast gate
  or al, 0x02  ; Set the second bit
  and al, 0xFE ; Clear the first bit
  out 0x92, al
 
  call test_a20
  test ax, ax
  jnz .ret

.fail:
  stc ; Sets the carry flag - failed to enable a20
  ret

.ret:
  clc
  ret

; Wait loop for the 0 bit of the keyboard controller to set
; Returns carry when timeout, else clear
keyboard_wait_bit_0:
  clc
  ; Set a timeout
  push cx
  mov cx, 0xFFFF
.loop:
  in al, 0x64
  test al, 1
  jnz .ret
  loop .loop
.fail:
  stc
.ret:
  pop cx
  ret

; Wait loop for the 1 bit of the keyboard controller to clear
; Returns carry when timeout, else clear
keyboard_wait_bit_1:
  clc
  ; Set a timeout
  push cx
  mov cx, 0xFFFF
.loop:
  in al, 0x64
  test al, 2
  jz .ret
  loop .loop
.fail:
  stc
.ret:
  pop cx
  ret

; Test if the A20 line is enabled. It should be only called in REAL mode!
; Returns:
;   ax is 1 if enabled
;   ax is 0 if disabled
test_a20:
  pushf
  push si
  push di
  push ds
  push es
  push bx

  ; Sets the source offset to the compare byte offset, and the destination offset to that plus 0x10
  mov ax, compare_byte
  mov si, ax
  mov di, ax
  add di, 0x10

  ; Sets ds to 0x0000 and es to 0xFFFF
  xor ax, ax
  mov ds, ax
  
  not ax 
  mov es, ax

  mov bl, byte [es:si] ; Save the byte at es:si

  xor ax, ax ; Clear ax for return if A20 is enabled
  
  mov byte [es:di], 0x00
  mov byte [ds:si], 0xFF 

  cmp byte [es:di], 0xFF
  je .ret

  mov ax, 1

.ret:
  mov byte [es:si], bl ; Restore the byte at es:si
  pop bx
  pop es
  pop ds
  pop di
  pop si
  popf
  ret

; Enters unreal mode. Will enter 32-bit protected mode and then return
; to real mode without reloading the segment selectors,
; which causes the proccessor to enter "unreal mode"
enter_unreal_mode:
  ; Enter 32-bit protected mode:

  ; Load a GDT
  lgdt [gdt_desc]

  ; Enable 32-bit protected mode in control register
  mov eax, cr0
  or eax, 1
  mov cr0, eax 

  ; Reload the segments
  xor ax, ax ; Need to check that works!
  mov ds, ax
  mov es, ax
  mov fs, ax 
  mov gs, ax

  ; Returns to unreal mode
  mov eax, cr0
  and eax, 0xFFFFFFFE
  mov cr0, eax
  jmp 0x0:.ret ; reload cs

.ret:
  ; Print unreal-mode entered message
  lea si, enter_unreal_mode_message
  mov al, 0x90 ; Black
  call print_line
  ret

; This will load the kernel from disk to memory (100000-1FFFFF).
; It should be called only in unreal mode.
; Returns:
;   Carry if failed, else carry clear
load_kernel:
  clc ; clear carry
  
  mov cx, 0x20 ; Read 64 segments and copy 32 times (64*32*0.5K=1024K)
  mov ah, 0x42 ; Function code
  lea si, dpa  ; load DPA
  mov dl, [boot_disk] ; Disk to read from (boot disk)
  ; Kernel offset:
  mov edi, 0x100000
  ; Data unreal segment:
  mov bx, 0x16
  mov es, bx
  ; Buffer real segment:
  mov bx, 0x7000
  mov gs, bx

.copy_loop:
  int 0x13
  jc .ret ; If error, return

  mov bx, 0x0
.copy_from_buffer_loop:
  ; Copy the memory from buffer to new location:
  mov dx, [gs:0x8000+bx]
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

.ret:
 ; Sets ax to the program offset pointed by the elf header (at bytes 24-27 of kernel):
  mov eax, [es:edi+24]
  mov [kernel_start], eax
  ret

; This will enter 32-bit mode from protected mode. This should be called
; from unreal mode. It will jump to main32
; Note: This function will NOT reload the segment selectors or GDT, as it
; assumes that unreal mode has already set them
enter_p_mode_from_unreal_mode:
  ; Notify BIOS of target processor mode
  mov ax, 0xEC00
  mov bl, 1
  int 0x15

  ; Go down a line
  lea si, new_line
  call print_line
  
  ; Print loading kernel
  lea si, kernel_loaded_message
  mov al, 0xD0 ; Black
  call print_line
  
  ; Enable 32-bit protected mode in control register
  mov eax, cr0
  or eax, 1
  mov cr0, eax


  ; Jump to 32-bit code
  jmp 0x08:main32


; 32-bit protected mode code
[BITS 32]

; This will setup a kernel stack (0x300000:0x3FFFFF), and jump to the kernel
main32:
  ; Setup kernel stack
  mov eax, 0x400000
  mov sp, ax
  mov bp, ax

  ; Jump to the kernel
  mov eax, [kernel_start]
  jmp eax

; Data:

; Text:
boot_message: db "First-stage booting, please wait...", 0x0
a20_error_message: db "CRITICAL ERROR: Unable to enable A20 line. Machine halted!", 0x0
a20_enabled_message: db "A20 line was successfuly enabled!", 0x0
enter_unreal_mode_message: db "Entered into unreal mode", 0x0
memcpy_error_message: db "CRITICAL ERROR: Unable to copy the kernel form disk. Machine halted!", 0x0
memcpy_success_message: db "Kernel copied from disk successfuly!", 0x0
kernel_loaded_message: db "Kernel loading, please wait...", 0x0

; GDT table (temporary - for entering protected mode):
gdt:

gdt_null:
  dq 0x0

; Base - 0x0
; Limit - 0xFFFFF
gdt_code:
  dw 0xFFFF    ; Limit (0-15)
  dw 0x0000    ; Base (0-15)
  db 0x00      ; Base (16-23)
  db 10011011b ; Accses byte
  db 11001111b ; Limit (16-19) + Flags
  db 0x00      ; Limit (24-31)

; Base - 0x0
; Limit - 0xFFFFF
gdt_data:
  dw 0xFFFF    ; Limit (0-15)
  dw 0x0000    ; Base (0-15)
  db 0x00      ; Base (16-23)
  db 10010011b ; Accses byte
  db 11001111b ; Limit (16-19) + Flags
  db 0x00      ; Limit (24-31)

gdt_end:

gdt_desc:
  dw gdt_end - gdt - 1
  dd gdt

; DPA - (disk address packet)
dpa:
  db 0x10      ; Size of DPA (16-bytes)
  db 0x0       ; Reserved
  dw 0x40      ; Read 64 sectors
  dw 0x8000    ; Buffer offset
  dw 0x7000    ; Buffer segment
  dd 0x2400    ; Starting read address LBA lower 32-bits
  dd 0x0       ; Starting read address LBA upper 16-bits (16-bits unused)

; Variables:

; Boot disk:
boot_disk: db 0x00

; Current line on screen:
screen_line: db 0x00

; Kernel starting address pointer (pointed by the elf header):
kernel_start: dq 0x00000000

; Compare byte for testing if A20 line is enabled:
compare_byte: db 0x00

; Print new line:
new_line: db 0x0
