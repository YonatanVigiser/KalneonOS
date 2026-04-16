global ap_init
global ap_init_end
extern ap_start

bits 16
ap_init:
  cli

  mov ax, cs
  shl eax, 4
  add eax, (idt - ap_init)
  lidt [eax]

  mov eax, 10100000b  ; Set the PAE and PGE bit.
  mov cr4, eax

  mov edi, [l4_table_frame]
  mov edx, edi        ; Point CR3 at the PML4.
  mov cr3, edx

  mov ecx, 0xC0000080 ; Read from the EFER MSR.
  rdmsr

  or eax, 0x00000900  ; Set the LME bit and NXE.
  wrmsr

  mov ecx, 0x277 ; Read from the PAT MSR.
  rdmsr

  mov eax, 0x00070106 ; Set the PAT value
  mov edx, 0x00070106
  wrmsr

  mov ebx, cr0        ; Activate long mode -
  or ebx, 0x80000001  ; - by enabling paging and protection simultaneously.
  mov cr0, ebx

  mov ax, cs
  shl eax, 4
  add eax, (gdt64.pointer - ap_init)
  lgdt [eax]
  jmp dword gdt64.code:long_mode_start

bits 64
long_mode_start:
  ; Reload segment registers
  xor ax, ax
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  mov ss, ax

  mov rsp, [rel stack_top_ptr]

  mov rax, ap_start
  call rax

  ; If kernel returns, halt
  cli
.loop:
  hlt
  jmp .loop


align 4
 idt:
  .length dw 0
  .base dd 0

; GDT for 64-bit mode
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64 ; new
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
.pointer:
    dw $ - gdt64 - 1
    dd gdt64

ap_init_end:

l4_table_frame: dd 0
stack_top_ptr: dq 0
cpu_id: dd 0
