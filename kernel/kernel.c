#include <stdint.h>

__attribute__((section(".text._start"), used, naked))
void _start(void) {
  __asm__ volatile (
    "call kernel_main\n"
    "ret\n"
  );
}

void kernel_main(void) {
  uint32_t c = 0;
  uint32_t* p = (uint32_t*) 0x200000;
  while (c <= 10) {
    *(p) = c;
    c++;
  }
}
