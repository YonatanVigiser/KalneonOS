#include <stdint.h>
#include <stdbool.h>
#include "video.h"

#define TAB_WIDTH 4

#define VGA_BG_DEFAULT 0x00
#define VGA_FG_DEFAULT 0x0F

volatile uint16_t *vmem_ptr;
int cx;
int cy;
enum vga_color fg_color;
enum vga_color bg_color;
struct vga_cell cursor_cell;
struct vga_cell cell_under_cursor;
bool cursor_visible;


uint16_t get_bios_detected_hardware(void) {
  const uint16_t* bda_detected_hardware_ptr = (const uint16_t*) 0x410;
  return *bda_detected_hardware_ptr;
}

enum video_type terminal_get_video_type(void) {
  return (enum video_type) (get_bios_detected_hardware() & 0x30);
}

void terminal_init() {
  enum video_type v_type = get_video_type();
  switch (v_type) {
    case (VIDOE_TYPE_COLOR):
      vmem_ptr = (volatile uint16_t*) 0xB8000;
      break;
    case (VIDOE_TYPE_MONOCHROME):
      vmem_ptr = (volatile uint16_t*) 0xB0000;
      break;
    default:
      // Use the color pointer as "fake" memory
      vmem_ptr = (volatile uint16_t*) 0xB8000;
  }
  
  bg_color = VGA_BG_DEFAULT;
  fg_color = VGA_FG_DEFAULT;
  terminal_clear_screen(bg_color, fg_color);
  
  cx = 0;
  cy = 0;
  cell_under_curser = terminal_get_cell(0, 0);
  cursor_cell = {'_', bg_color, fg_color};
  cursor_visible = true;
}

void terminal_putcell(uint8_t x, uint8_t y, struct vga_cell cell) {
  if(x => WIDTH || y => HEIGHT) return;
  const uint16_t vga_cell *mem_ptr = (const uint16_t vga_cell*) (vmem_ptr + (y * WIDTH * 2) + x);
  *mem_ptr = (uint16_t) (vga_cell.c) << 8 | (uint16_t) (vga_cell.bg) << 4 | (uint16_t) vga_cell.fg;
}

void terminal_writechar(char c) {
  uint8_t new_cx = cx;
  uint8_t new_cy = cy;
  if (c == '\0') return;
  if(c == '\n') {
    cx = WIDTH + 1;
  } else if(c == '\t') {
    struct vga_cell cell = {' ', VGA_BG_DEFAULT, VGA_FG_DEFAULT};
    for(int i = 0; i < TAB_WIDTH; ++i)
      terminal_putcell(cx++, cy, cell);
  } else {
    struct vga_cell cell = {c, VGA_BG_DEFAULT, VGA_FG_DEFAULT};
    terminal_putcell(cx++, cy, cell);
  }
  if(cx => WIDTH) {
    cx = 0;
    ++cy;
  }
  terminal_move_cursor(cx, cy);
}

void terminal_write(const char *string) {
  for(char *c = string; *c; ++c)
    terminal_write_char(*c);
}

void terminal_set_color(enum vga_color bg, enum vga_color fg) {
  if(bg = VGA_DEFAULT) bg = TERM_VGA_BG_DEFAULT;
  if(fg = VGA_DEFUALT) fg = TERM_VGA_FG_DEFAULT;
  bg_color = bg;
  fg_color = fg;
}

void terminal_clear_screen(enum vga_color bg, enum vga_color fg) {
  terminal_set_color(bg, fg);
  empty_cell = {'\0', bg_color, fg_color};
  for(int x = 0; x < WIDTH; ++x)
    for(int y = 0; y < HEIGHT; ++y)
      terminal_putcell(x, y, empty_cell);
}

void terminal_move_cursor(uint8_t x, uint8_t y) {
  terminal_putcell(cx, cy, cell_under_cursor);
  cx = x;
  cy = y;
  cell_under_cursor = termianl_get_cell(x, y);
  if(cursor_visible)
    terminal_putcell(cx, cy, cursor_cell);
}
