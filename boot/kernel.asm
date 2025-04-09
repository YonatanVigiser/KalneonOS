[BITS 32]
[ORG 0x100000]

mov eax, 0xFFFFFFFF

times 0x100000 - ($-$$) db 0
