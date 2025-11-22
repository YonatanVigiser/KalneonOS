extern main

section .text._start
[BITS 32]
global _start
_start:
  ; Multiboot2 provides:
  ; eax = magic value (0x36d76289)
  ; ebx = physical address of multiboot info structure

  ; Disable interrupts
  cli

  ; Save multiboot info
  mov edi, eax
  mov esi, ebx

  ; Set up initial stack
  mov esp, stack_top

  ; Verify we're running on a CPU that supports long mode
  call check_cpuid
  call check_long_mode

  ; Load 64-bit GDT while still in pure 32-bit mode
  lgdt [gdt64_desc]

  ; Set up paging for long mode
  call setup_page_tables
  call enable_paging

  ; Now we're in compatibility mode, perform far jump to enter 64-bit mode
  ; Update CS to the 64-bit code segment
  jmp long_mode_start

; Check if CPUID is supported
check_cpuid:
  pushfd
  pop eax
  mov ecx, eax
  xor eax, 1 << 21
  push eax
  popfd
  pushfd
  pop eax
  push ecx
  popfd
  xor eax, ecx
  je .no_cpuid
  ret
.no_cpuid:
  mov esi, error_no_cpuid
  jmp error

; Check if long mode is available
check_long_mode:
  mov eax, 0x80000000
  cpuid
  cmp eax, 0x80000001
  jb .no_long_mode

  mov eax, 0x80000001
  cpuid
  test edx, 1 << 29
  jz .no_long_mode
  ret
.no_long_mode:
  mov esi, error_no_long_mode
  jmp error

; Set up identity paging for the first 1GB using 2MB huge pages
setup_page_tables:
  ; Clear page tables (bss section might not be zeroed)
  mov edi, p4_table
  mov ecx, 3 * 4096 / 4  ; 3 tables, 4096 bytes each, divide by 4 for dwords
  xor eax, eax
  rep stosd

  ; Map P4[0] -> P3 (use edi which already points to p4_table)
  mov edi, p4_table
  mov eax, p3_table
  or eax, 0b11  ; present + writable
  stosd  ; Store eax at [edi] and increment edi
  xor eax, eax
  stosd  ; Store upper 32 bits (zero for 32-bit addresses)

  ; Map P3[0] -> P2
  mov edi, p3_table
  mov eax, p2_table
  or eax, 0b11  ; present + writable
  stosd
  xor eax, eax
  stosd

  ; Map P2 entries to 512 × 2MB pages = 1GB
  mov edi, p2_table
  mov ecx, 512  ; Loop counter
  xor edx, edx  ; Page number counter
  mov ebx, 0b10000011  ; present + writable + huge page
.map_p2_table:
  mov eax, edx
  shl eax, 21  ; multiply by 2MB (2^21)
  or eax, ebx
  stosd  ; Store the entry (lower 32 bits)
  xor eax, eax
  stosd  ; Store upper 32 bits (zero)

  inc edx
  loop .map_p2_table

  ret

; Enable paging and long mode
enable_paging:
  ; Load P4 table into CR3
  mov eax, p4_table
  mov cr3, eax

  ; Enable PAE (Physical Address Extension)
  mov eax, cr4
  or eax, 1 << 5
  mov cr4, eax

  ; Enable long mode in EFER MSR
  mov ecx, 0xC0000080
  rdmsr
  or eax, 1 << 8
  wrmsr

  ; Enable paging
  mov eax, cr0
  or eax, 1 << 31
  mov cr0, eax

  ret

; Error handler - prints error message and halts
; Input: si = pointer to error message string
error:
  mov edi, 0xb8000  ; VGA text buffer
  mov ah, 0x4f      ; White text on red background
.loop:
  lodsb             ; Load byte from [si] into al
  test al, al       ; Check for null terminator
  jz .done
  mov [edi], ax     ; Write character + attribute
  add edi, 2        ; Move to next character position
  jmp .loop
.done:
  cli
  hlt

; 64-bit entry point
[BITS 64]
long_mode_start:
  ; Reload segment registers
  mov ax, 0x10
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  mov ss, ax

  call main

  ; If kernel returns, halt
  cli
.loop:
  hlt
  jmp .loop

; GDT for 64-bit mode
align 8
gdt64:
  dq 0x0000000000000000  ; Null descriptor
  dq 0x00AF9A000000FFFF  ; 64-bit code segment
  dq 0x00CF92000000FFFF  ; 64-bit data segment
gdt64_end:

gdt64_desc:
  dw gdt64_end - gdt64 - 1  ; Limit
  dd gdt64                   ; Base (32-bit in protected mode)

; Error messages
error_no_cpuid: db "ERROR: CPUID not supported", 0
error_no_long_mode: db "ERROR: Long mode not available", 0

; Page tables (must be page-aligned)
section .bss
align 4096
p4_table:
  resb 4096
p3_table:
  resb 4096
p2_table:
  resb 4096

; Stack
section .bss.stack
align 16
stack:
  resb 0x10000
stack_top:
