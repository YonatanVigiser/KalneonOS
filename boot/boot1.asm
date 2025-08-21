[BITS 16]
[ORG 0x1000]

; The zero-stage bootloader will jump here.
; Will print another booting message, enable A20,
; and enter 32-bit protected mode:
main16:
  mov byte [boot_disk], dl ; Save the boot disk number
  mov word [partition_table_ptr], si ; Save the partition table pointer
  mov eax, dword [si+0x08]
  mov dword [partition_lba_sector], eax

  call init_screen
 
  ; Enable A20 line
  call enable_a20
  jc .a20_error

  ; Print A20 enabled message
  lea si, a20_enabled_message
  call print_line

  ; Enter unreal mode
  call enter_unreal_mode
  
  ; Generate boot info
  call build_boot_info_block
  jc .build_info_error

  ; Load the kernel
  call load_kernel
  jc .memcpy_error

  ; Print kernel copy message
  lea si, memcpy_success_message
  call print_line

  ; Enter protected mode
  jmp enter_p_mode_from_unreal_mode

.a20_error:
  lea si, a20_error_message
  mov al, 0x40 ; Red background, black text
  call print_line_color
  jmp .hlt

.build_info_error:
  lea si, build_info_error_message
  mov al, 0x40 ; Red background, black text
  call print_line_color
  jmp .hlt

.memcpy_error:
  lea si, memcpy_error_message
  mov al, 0x40 ; Red background, black text
  call print_line_color
  jmp .hlt

.hlt:
  hlt
  jmp .hlt

; Clear the screen, sets a background color,
; and print boot message
init_screen:
  ; Set the video mode to 0x03
  mov ah, 0x00
  mov al, 0x03
  int 0x10

  ; Clear the screen and set background color
  mov ah, 0x06
  mov al, 0x00              ; Clear all
  mov bh, byte [text_color] ; Background color
  and bh, 0xf0              ; Foreground color (unused)
  mov ch, 0x00              ; y=0
  mov cl, 0x00              ; x=0
  mov dh, 0x19              ; y=25
  mov dl, 0x50              ; x=80
  int 0x10

  ; Disable cursor
  mov ah, 0x01
  mov ch, 0x3F
  int 0x10

  ; Print welcome message:
  lea si, welcome_message
  call print_line

  ; Print new line:
  lea si, new_line
  call print_line

  ; Print booting message:
  lea si, boot_message
  call print_line

.ret:
  ret

; Print a line in the default color to the screen
; Params:
;   Pointer to string at ds:si
print_line:
  push ax

  mov al, byte [text_color]
  call print_line_color

.ret:
  pop ax
  ret

; Print a line in a specific color to the screen
; Params:
;   Pointer to string at ds:si
;   al - color
print_line_color:
  push bx
  push cx
  push dx

  ; Check if printing is enabled
  mov bl, byte [print_enable]
  test bl, bl
  jz .ret

  ; Print only one char at a time
  mov cx, 1
  
  ; Page num is 0
  mov bh, 0

  ; Set the text color
  mov bl, al
  
  ; Set the rows and columns
  mov dh, byte [screen_line]
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
  inc dl
  jmp .print_loop

.ret:
  mov ah, byte [screen_line]
  add ah, 1
  mov byte [screen_line], ah
  pop dx
  pop cx
  pop bx
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
  mov ax, 0x10
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
  ; Reload the segments
  xor ax, ax
  mov ds, ax
  mov es, ax
  mov fs, ax 
  mov gs, ax

  ; Print unreal-mode entered message
  lea si, enter_unreal_mode_message
  call print_line
  ret

; This function gather information about the machine and generate the
; boot info section, used to pass info from the bootloader to the kernel.
; See: docs/boot-info/general for more info about the boot info section.
build_boot_info_block:
  lea si, BOOT_INFO_BLOCK_P

  mov dword [si], 0x594F5649 ; magic bits: "YOVI"
  mov word [si+0x4], 0x1 ; Version 1

  mov dword [si+0x10], KERNEL_P
  mov dword [si+0x18], KERNEL_SIZE_BYTES

  mov byte [si+0xE], boot_disk

; Detects CPUID suppport:
.cpuid:
  pushfd ; Save EFLAGS
  pushfd ; Store EFLAGS
  xor dword [esp], 0x00200000 ; Invert the ID bit in the stored EFLAGS
  popfd ; Load the modified EFLGAS
  pushfd ; Store EFLAGS again (modified  - but ID bit may not be changed)
  pop eax ; Load EFLAGS to eax
  xor eax, [esp] ; Store whichever bits where changed in eax
  popfd ; Restore EFLAGS
  and eax, 0x0020000 ; Check if the ID bit was changed (if so then CPUID is supported, else not)
  jz .detect_mem
  ; Store the result:
  mov al, byte [si+0x6]
  or al, 0x2
  mov byte [si+0x6], al

; Detects memory and build a memory map (See: docs/boot-info/memory-map for more info)
.detect_mem:
  xor eax, eax
  mov es, ax
  ; Set the first entry pointer:
  mov ax, si
  add ax, 0x20 ; Mem map offset in boot info block
  mov di, ax
  xor ebx, ebx
  mov edx, 0x534D4150 ; Magic value
  mov cx, 0x18 ; Length of the entry
  mov ax, 0xE820 ; Function code
  int 0x15 ; Call the first int
  jc .mem_detection_failed
  cmp eax, edx ; Test if eax contains the magic value
  jne .mem_detection_failed
  
  xor ax, ax ; Counter
  test cx, 0x18 ; Test if EAB is supported
  jne .mem_detection_loop 

  ; Indicate EAB support for mem map:
  mov dl, byte [si+0x6]
  or dl, 0x1
  mov byte [si+0x6], dl

.mem_detection_loop:
  inc ax ; Increament counter
  mov edx, 0x534D4150 ; Magic value
  add di, 0x18 ; Increament pointer
  push ax ; Save the counter
  xor eax, eax
  mov ax, 0xE820 ; Function code
  mov cx, 0x18 ; Length of the entry
  int 0x15
  pop ax ; Restore the counter
  jc .detect_mem_end ; The end of the list was reached 
  test bx, bx
  jz .detect_mem_end ; The end of the list was reached
  jmp .mem_detection_loop

.detect_mem_end:
  clc ; Clears the carry flag from previous detection
  ; Save map entrys number:
  mov byte [si+0xF], al
  jmp .ret
.mem_detection_failed:
  stc
.ret:
  ret

; This will load the kernel from disk to memory (100000-1FFFFF).
; It should be called only in unreal mode.
; Returns:
;   Carry if failed, else carry clear
load_kernel:
  clc ; clear carry

  ; Real-mode segment 
  xor ax, ax
  mov gs, ax

  ; Load the LBA sector
  mov eax, dword [partition_lba_sector]
  add eax, 0x12 ; The 18'th sector in the partition
  mov dword [dpa + 0x08], eax
  xor eax, eax

  mov cx, 0x20 ; Read 32 times 32K (32K*32=1024K)
  mov ah, 0x42 ; Function code
  lea si, dpa  ; Load DPA
  mov dl, byte [boot_disk] ; Disk to read from (boot disk)
  mov edi, KERNEL_P ; Kernel offset

.copy_loop:
  clc ; Clears the overflow flag
  int 0x13
  mov ah, 0x42
  jc .ret ; If error, return
  
  mov bx, KERNEL_COPY_BUFF_P ; Buffer starting address

.copy_from_buffer_loop:
  ; Copy the memory from buffer to new location:
  mov dh, byte [gs:bx]
  mov byte [edi], dh

  ; Loop:
  add edi, 1
  inc bx
  test bx, bx ; bx will overflow to zero at the end
  jnz .copy_from_buffer_loop

  ; Add the reading starting address 
  ; 0x8000 (64*512=32,768):

  mov ebx, [si+8]
  add ebx, 0x40
  mov [si+8], ebx
  mov ebx, [si+8]

  loop .copy_loop

.ret:
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
  call print_line
  
  ; Enable 32-bit protected mode in control register
  mov eax, cr0
  or eax, 1
  mov cr0, eax

  ; Reload segments
  mov ax, 0x10
  mov ds, ax
  mov es, ax
  mov fs, ax 
  mov gs, ax
  mov ss, ax

  ; Setup protected mode stack
  mov eax, PROTECTED_MODE_STACK_P 
  mov esp, eax
  mov ebp, eax

  ; Jump to 32-bit code
  jmp 0x08:main32


; 32-bit protected mode code
[BITS 32]

; setup a kernel stack (0x200000:0x2FFFFF), and jump to the kernel
main32:
  ; Setup kernel stack
  mov eax, KERNEL_STACK_P
  mov esp, eax
  mov ebp, eax

  ; Push the boot info block to the stack
  push BOOT_INFO_BLOCK_P

  ; Jump to the kernel
  mov eax, KERNEL_P
  call eax

; In case the kernel returns, halt the machine:
.hlt:
  hlt
  jmp .hlt

; Data:

; Text:
welcome_message: db "Welcome to KalneonOS!", 0x0
boot_message: db "Booting, please wait...", 0x0
a20_error_message: db "CRITICAL ERROR: Unable to enable A20 line. Machine halted!", 0x0
a20_enabled_message: db "A20 line was successfuly enabled", 0x0
enter_unreal_mode_message: db "Entered into unreal mode", 0x0
build_info_error_message: db "CRITICAL ERROR: Unable to create the Boot Info Block. Machine halted!", 0x0
memcpy_error_message: db "CRITICAL ERROR: Unable to copy the kernel form disk. Machine halted!", 0x0
memcpy_success_message: db "Kernel copied from disk successfuly", 0x0
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
  dw KERNEL_COPY_BUFF_P ; Buffer offset
  dw 0x0000    ; Buffer segment
  dq 0x00      ; Start reading from LBA sector num. Fill dynamiclly

; Variables:

; Boot disk:
boot_disk: db 0x00

; Partition table pointer
partition_table_ptr: dw 0x0000

; partition LBA sector
partition_lba_sector: dq 0x0

; CPUID support
cpuid_supported: db 0x00

; Current line on screen:
screen_line: db 0x00

; Compare byte for testing if A20 line is enabled:
compare_byte: db 0x00

; New line:
new_line: db 0x0

; Settings:

; Settings padding (max 64 bytes)
times 0x2200 - 0x40 - ($-$$) db 0

print_enable: db 0x1 ; Zero disabled, else enabled 

text_color: db 0x10  ; Background color (default: light-blue), text color (default: black)

; File padding:
times 0x2200 - ($-$$) db 0

; Constants:

; Pointer to the boot info block location in memory:
BOOT_INFO_BLOCK_P equ 0x3200

; Pointer to a 64K buffer for copying the kernel
KERNEL_COPY_BUFF_P equ 0x8000

; Pointer to the kernel copy destination in memory:
KERNEL_P equ 0x100000

; Size of kernel in bytes:
KERNEL_SIZE_BYTES equ 0x100000

; Pointer to the kernek stack in memory:
KERNEL_STACK_P equ 0x400000

; Poiner to the protected mode stack:
PROTECTED_MODE_STACK_P equ 0x20000
