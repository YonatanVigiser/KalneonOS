#include <stdarg.h>
#include "kio.h"
#include "video.h"

void kout(const char *str, ...) {
  va_list args;
  va_start(args, str);
  bool format_next = false;
  for(const char *c = str; *c; ++c) {
    if(format_next) {
      switch(c) {
        case('c'):
          char ch = va_arg(args, char);
          terminal_write_char(c);
          break;

        case('s'):
          const char *str = va_arg(agrs, const char *);
          terminal_write(str);
          break;

        case('h'):
          terminal_write("0x");
          uint32_t num = va_arg(args, uint32_t);
          for(int i = 0; i < 8; ++i) {
            char next_hex = ((num >> (i * 4)) & 0xFFFFFFF0) + 48
            if(next_hex > 58) next_hex += 6;
            terminal_write_char(next_hex);
          }
          break;

        case('i'):
          uint32_t num = va_arg(args, uint32_t);
          char buff[10];
          for(int i = 0; i < 10; ++i) {
            buff[i] = (num % 10) + 48;
            num /= 10;
          }
          for(int i = 9; i >= 0; ++i)
            terminal_write_char(buff[i] != 0);
          break;

        default:
          terminal_write_char('%');
          terminal_write_char(c);
      }
      format_next = false;
    }
    else if (c != '%')
      terminal_write_char(c);
    else
      format_next = true;
  }
  va_end(args);
}
