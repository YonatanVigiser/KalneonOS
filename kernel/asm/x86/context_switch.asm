global context_switch
context_switch:
  ; Save context on current stack
  pushfd
  push eax
  push ebx
  push ecx
  push edx
  push ebp
  push esi
  push edi
  
  ; Save the updated stack pointer to old_stack_ptr
  mov eax, [esp + 36]   ; Get old_stack_ptr
  mov [eax], esp        ; Save current esp
  
  ; Load new stack pointer
  mov eax, [esp + 40]   ; Get new_stack_ptr
  mov esp, eax
  
  ; Restore context from new stack
  pop edi
  pop esi
  pop ebp
  pop edx
  pop ecx
  pop ebx
  pop eax
  popfd
    
  ret

global fake_thread_entry_stack
fake_thread_entry_stack:
  mov eax, [esp + 4]    ; Get pointer to stack_ptr
  mov ecx, [esp + 8]    ; Get entry point
  mov edx, [eax]        ; Get current stack pointer value

  ; Allocate space for all registers
  sub edx, 36

  ; Set up stack frame that context_switch expects
  mov dword [edx], 0              ; EFLAGS = 0
  mov dword [edx + 4], 0          ; EAX = 0
  mov dword [edx + 8], 0          ; ECX = 0
  mov dword [edx + 12], 0         ; EDX = 0
  mov dword [edx + 16], 0         ; EBX = 0
  mov dword [edx + 20], edx + 36  ; EBP = 
  mov dword [edx + 24], 0         ; ESI = 0
  mov dword [edx + 28], 0         ; EDI = 0
  mov [edx + 32], ecx             ; Return address = entry point

  ; Update stack pointer
  mov [eax], edx

  ret

