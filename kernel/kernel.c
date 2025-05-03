#include <stdint.h>

void _start(void) {
  __asm__ volatile (
    "call kernel_main\n"
    "ret\n"
  );
}

void kernel_main(void) {

}
