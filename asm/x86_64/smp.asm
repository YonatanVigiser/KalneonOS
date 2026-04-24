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

  mov eax, cs
  shl eax, 4
  mov edi, [eax + l4_table - ap_init]
  mov edx, edi
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

  mov eax, cs
  shl eax, 4
  mov ebx, eax
  mov edx, eax ; Save phyiscal start address
  add ebx, (gdt64 - ap_init)
  mov [eax + (gdt64.pointer - ap_init + 2)], ebx ; Change the base addr of the GDT at runtime
  add eax, (gdt64.pointer - ap_init)
  lgdt [eax]
  
  mov eax, edx
  add edx, (long_mode_start - ap_init)
  mov [eax + far_target - ap_init], edx
  o32 jmp far [eax + far_target - ap_init]

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

  ; Pass the cpu id
  xor rdi, rdi
  mov edi, dword [rel cpu_id]

  mov rax, ap_start
  call rax

  ; If kernel returns, halt
  cli
.loop:
  hlt
  jmp .loop

far_target:
    dd 0          ; offset
    dw gdt64.code ; segment selector

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
    dd 0

temp_stack: dq 0

align 8
; Params (refer to the ApCoreData struct defined in smp.rs):
stack_top_ptr: dq 0
l4_table: dd 0
cpu_id: dd 0

ap_init_end:
