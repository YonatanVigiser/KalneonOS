; Define generic stub generation helper macros
%macro isr_err_stub 1
isr_stub_%+%1:
  cli
  push %1
  pusha
  push ds
  push es
  push fs
  push gs
  push esp
  call intterupts_handler
  pop eax
  pop gs
  pop fs
  pop es
  pop ds
  popa
  iret
%endmacro

%macro isr_no_err_stub 1
isr_stub_%+%1:
  cli
  push 0x0000
  push %1
  pusha
  push ds
  push es
  push fs
  push gs
  push esp
  call intterupts_handler
  pop eax
  pop gs
  pop fs
  pop es
  pop ds
  popa
  iret
%endmacro

; Call the kernel's general CPU exceptions handler (defined in rust)
extern intterupts_handler

; Define the exception handlers
isr_no_err_stub 0
isr_no_err_stub 1
isr_no_err_stub 2
isr_no_err_stub 3
isr_no_err_stub 4
isr_no_err_stub 5
isr_no_err_stub 6
isr_no_err_stub 7
; 8 - Double Fault, custom handler.
isr_no_err_stub 9
isr_err_stub    10
isr_err_stub    11
isr_err_stub    12
isr_err_stub    13
isr_err_stub    14
isr_no_err_stub 15
isr_no_err_stub 16
isr_err_stub    17
isr_no_err_stub 18
isr_no_err_stub 19
isr_no_err_stub 20
isr_no_err_stub 21
isr_no_err_stub 22
isr_no_err_stub 23
isr_no_err_stub 24
isr_no_err_stub 25
isr_no_err_stub 26
isr_no_err_stub 27
isr_no_err_stub 28
isr_no_err_stub 29
isr_err_stub    30
isr_no_err_stub 31
isr_no_err_stub 32
isr_no_err_stub 33
isr_no_err_stub 34
isr_no_err_stub 35
isr_no_err_stub 36
isr_no_err_stub 37
isr_no_err_stub 38
isr_no_err_stub 39
isr_no_err_stub 40
isr_no_err_stub 41
isr_no_err_stub 42
isr_no_err_stub 43
isr_no_err_stub 44
isr_no_err_stub 45
isr_no_err_stub 46
isr_no_err_stub 47

isr_stub_8:
  jmp isr_stub_8

; Define a stub table for ease of use
global isr_stub_table
isr_stub_table:
%assign i 0
%rep 48
  dd isr_stub_%+i
%assign i i+1
%endrep
