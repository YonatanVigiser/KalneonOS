/*
#include <stdint.h>
#include "video.h"

#define WIDTH=80
#define HEIGHT=25

volatile uint16_t *vmem_ptr;

uint16_t get_bios_detected_hardware() {
  const uint16_t* bda_detected_hardware_ptr = (const uint16_t*) 0x410;
  return *bda_detected_hardware_ptr;
}

enum video_type get_video_type () {
  return (enum video_type) (get_bios_detected_hardware() & 0x30);
}

void init_terminal() {
  enum video_type v_type = get_video_type();
  switch (v_type) {
    case (VIDOE_TYPE_COLOR):
      vmem_ptr = (volatile uint16_t*) 0xB8000;
      break
    case (VIDOE_TYPE_MONOCHROME):
      vmem_ptr = (volatile uint16_t*) 0xB0000;
      break;
    default:
      // Use the color pointer as "fake" memory
      vmem_ptr = (volatile uint16_t*) 0xB8000;
  }
}

void terminal_putchar(uint8_t x, uint8_t y, char c, 
    enum vga_color bg_color, enum vga_color color) {
  if(x < 0 || y < 0 || x > WIDTH || y > HEIGHT) return;
  const uint16_t *mem_ptr = (const uint16_t*) vmem_ptr + (y * WIDTH * 2) + x;
  *mem_ptr = (c << 8) | (bg_color << 4) | color;
}

void terminal_put_string(uint8_t x, uint8_t y, const char *string,
    enum vga_color bg_color, enum vga_color color) {

}

*/
