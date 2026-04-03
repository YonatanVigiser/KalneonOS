extern main
extern __stack_top
extern __boot_stack_top
extern __bss_start
extern __bss_end
global _start

section .text._start
bits 32
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
  mov esp, __boot_stack_top

  ; Check if already in long mode
  mov ecx, 0xC0000080
  rdmsr
  test eax, 1 << 10      ; LMA = Long Mode Active
  jnz already_in_long_mode

  ; Verify we're running on a CPU that supports long mode
  call check_cpuid
  call check_long_mode

  ; Set up paging for long mode
  call setup_page_tables
  jmp enter_long_mode

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

; Set up identity paging for the first 1GB, and also map the first 1G to high memory
setup_page_tables:
  push edi
  ; Clear page tables (bss section might not be zeroed)
  mov edi, p4_table
  mov ecx, 3 * 4096 / 4
  xor eax, eax
  rep stosd

  ; Identity map: P4[0] -> p3_low_table
  mov edi, p4_table
  mov eax, p3_low_table
  or  eax, 0b11
  mov [edi], eax
  mov dword [edi + 4], 0

  ; P3[0] -> 1GiB huge page
  mov edi, p3_low_table
  mov eax, 0b10000011 ; present + writable + huge
  mov [edi], eax
  mov dword [edi + 4], 0 ; phys addr 0

  ; Kernel: P4[511] -> p3_kernel_table -> 1GiB huge page
  mov edi, p4_table + 511 * 8
  mov eax, p3_kernel_table
  or  eax, 0b11
  mov [edi], eax
  mov dword [edi + 4], 0

  ; P3[510] -> 1GiB huge page
  mov edi, p3_kernel_table + 510 * 8
  mov eax, 0b10000011 ; present + writable + huge
  mov [edi], eax
  mov dword [edi + 4], 0 ; phys addr 0

  pop edi
  ret

; Enable paging and long mode
enter_long_mode:
  mov eax, p4_table
  mov cr3, eax

  ; Enable PAE
  mov eax, cr4
  or eax, 1 << 5  ; PAE (bit 5)
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

  lgdt [gdt64.pointer]

  jmp dword gdt64.code:long_mode_start

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

bits 64
; Build page tables, load a GDT and jump to long_mode_start
already_in_long_mode:
  mov rsp, __boot_stack_top
  push rdi
  push rsi

  ; Clear page tables (bss section might not be zeroed)
  mov rdi, p4_table
  mov rcx, 3 * 4096 / 8
  xor rax, rax
  rep stosq

  ; Identity map: P4[0] -> p3_low_table
  mov rdi, p4_table
  mov rax, p3_low_table
  or  rax, 0b11
  mov [rdi], rax

  ; P3[0] -> 1GiB huge page
  mov rdi, p3_low_table
  mov qword [rdi], 0b10000011 ; present + writable + huge + phys addr 0

  ; Kernel: P4[511] -> p3_kernel_table -> 1GiB huge page
  mov rdi, p4_table + 511 * 8
  mov rax, p3_kernel_table
  or  rax, 0b11
  mov [rdi], rax

  ; P3[510] -> 1GiB huge page
  mov rdi, p3_kernel_table + 510 * 8
  mov qword [rdi], 0b10000011 ; present + writable + huge + phys addr 0

  mov rax, p4_table
  mov cr3, rax

  pop rsi
  pop rdi
  jmp long_mode_start

long_mode_start:
  ; Reload segment registers
  xor ax, ax
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  mov ss, ax

  mov rsp, __stack_top

  push rdi
  push rsi
  ; Zero .bss
  mov rdi, __bss_start
  mov rcx, __bss_end
  sub rcx, rdi
  xor al, al
  rep stosb
  pop rsi
  pop rdi

  mov rax, main
  call rax

  ; If kernel returns, halt
  cli
.loop:
  hlt
  jmp .loop


section .boot.rodata
; GDT for 64-bit mode
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64 ; new
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
.pointer:
    dw $ - gdt64 - 1
    dd gdt64

; Error messages
error_no_cpuid: db "ERROR: CPUID not supported", 0
error_no_long_mode: db "ERROR: Long mode not available", 0

; Page tables (must be page-aligned)
section .boot.bss
align 4096
p4_table:
  resb 4096
p3_low_table:
  resb 4096
p3_kernel_table:
  resb 4096
