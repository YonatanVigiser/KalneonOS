global context_switch
context_switch:
  ; Save registers on current stack
  push ebx
  push edi
  push esi
  
  ; Save the updated stack pointer to old_stack_ptr
  mov eax, [esp + 16]   ; Get old_stack_ptr
  mov [eax], esp        ; Save current esp
  
  ; Load new stack pointer
  mov eax, [esp + 20]   ; Get new_stack_ptr
  mov esp, eax
  
  ; Restore registers from new stack
  pop esi
  pop edi
  pop ebx
    
  ret

global fake_thread_entry_stack
fake_thread_entry_stack:
  mov eax, [esp + 4]    ; Get pointer to stack_ptr
  mov ecx, [esp + 8]    ; Get entry point
  mov edx, [eax]        ; Get current stack pointer value

  ; Allocate space for 4 dwords
  sub edx, 16

  ; Set up stack frame that context_switch expects
  mov dword [edx], 0       ; ESI = 0
  mov dword [edx + 4], 0   ; EDI = 0
  mov dword [edx + 8], 0   ; EBX = 0
  mov [edx + 12], ecx      ; Return address = entry point

  ; Update stack_ptr to point to the base of our frame
  mov [eax], edx

  ret

