#include <stdint.h>
#include "video.h"

void kernel_main(void);

__attribute__((naked, noreturn))
void _start(void) {
    __asm__ volatile (
        "call kernel_main\n"
        "hlt\n"
        "jmp .-2\n"
    );
}

void kernel_main(void) {
}
