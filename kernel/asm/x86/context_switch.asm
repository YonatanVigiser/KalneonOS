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
  mov edx, [eax]        ; Get stack pointer value

  ; Set up stack frame that context_switch expects
  mov dword [edx - 4], ecx       ; Return address = entry point
  mov dword [edx - 8], 0     ; EFLAGS = 0
  mov dword [edx - 12], 0     ; EAX = 0
  mov dword [edx - 16], 0    ; EBX = 0
  mov dword [edx - 20], 0    ; ECX = 0
  mov dword [edx - 24], 0    ; EDX = 0
  mov dword [edx - 28], edx  ; EBP = stack_ptr
  mov dword [edx - 32], 0    ; ESI = 0
  mov dword [edx - 36], 0    ; EDI = 0

  ; Update stack pointer
  sub edx, 36
  mov [eax], edx

  ret

