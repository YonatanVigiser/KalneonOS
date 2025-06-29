#include <stdint.h>
#include <stdbool.h>
#include "video.h"

volatile uint16_t *vmem_ptr;
uint8_t cx;
uint8_t cy;
enum vga_color fg_color;
enum vga_color bg_color;
struct vga_cell cursor_cell;
struct vga_cell cell_under_cursor;
bool cursor_visible;

#define VGA_BG_DEFAULT 0x0
#define VGA_FG_DEFAULT 0xF

uint16_t get_bios_detected_hardware(void) {
  const uint16_t* bda_detected_hardware_ptr = (const uint16_t*) 0x410;
  return *bda_detected_hardware_ptr;
}

enum video_type terminal_get_video_type(void) {
  return (enum video_type) (get_bios_detected_hardware() & 0x30);
}

void terminal_init(void) {
  enum video_type v_type = terminal_get_video_type();
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

  cell_under_cursor = terminal_get_cell(0, 0);
  cursor_cell.c = '_';
  cursor_cell.bg = bg_color;
  cursor_cell.fg = fg_color;
  cursor_visible = true;
  terminal_move_cursor(0, 0);
}

void terminal_put_cell(uint8_t x, uint8_t y, struct vga_cell cell) {
  if(x >= TERM_WIDTH || y >= TERM_HEIGHT) return;
  volatile uint16_t *mem_ptr = (volatile uint16_t *) vmem_ptr + (y * TERM_WIDTH + x);
  *mem_ptr = (uint16_t) (cell.bg) << 12 | (uint16_t) (cell.fg) << 8 | (uint16_t) cell.c;
}

struct vga_cell terminal_get_cell(uint8_t x, uint8_t y) {
  struct vga_cell null_cell = {'\0', VGA_BLACK, VGA_BLACK};
  if(x >= TERM_WIDTH || y >= TERM_HEIGHT) return null_cell;
  volatile uint16_t *mem_ptr = (volatile uint16_t *) vmem_ptr + (y * TERM_WIDTH + x);
  struct vga_cell cell;
  cell.c = (char) (*mem_ptr & 0x00FF);
  cell.bg = (enum vga_color) (*mem_ptr & 0xF000) >> 12;
  cell.fg = (enum vga_color) (*mem_ptr & 0x0F00) >> 8;
  return cell;
}

void terminal_write_char(char c) {
  if (c == '\0') return;
  if(c == '\n') {
    // This will then reset cx, and inc cy
    cx = TERM_WIDTH;
  } else if(c == '\t') {
    struct vga_cell cell = {' ', bg_color, fg_color};
    for(int i = 0; i < TAB_WIDTH; ++i)
      terminal_put_cell(cx++, cy, cell);
  } else {
    struct vga_cell cell = {c, bg_color, fg_color};
    terminal_put_cell(cx++, cy, cell);
  }
  if(cx >= TERM_WIDTH) {
    cx = 0;
    ++cy;
  }
  terminal_move_cursor(cx, cy);
}

void terminal_write(char *string) {
  for(char *c = string; *c; ++c)
    terminal_write_char(*c);
}

void terminal_set_color(enum vga_color bg, enum vga_color fg) {
  if(bg == VGA_DEFAULT) bg = VGA_BG_DEFAULT;
  if(fg == VGA_DEFAULT) fg = VGA_FG_DEFAULT;
  bg_color = bg;
  fg_color = fg;
}

enum vga_color terminal_get_bg_color(void) {
  return bg_color;
}

enum vga_color terminal_get_fg_color(void) {
  return fg_color;
}

void terminal_clear_screen(enum vga_color bg, enum vga_color fg) {
  terminal_set_color(bg, fg);
  struct vga_cell empty_cell = {'\0', bg_color, fg_color};
  for(int x = 0; x < TERM_WIDTH; ++x)
    for(int y = 0; y < TERM_HEIGHT; ++y)
      terminal_put_cell(x, y, empty_cell);
}

void terminal_move_cursor(uint8_t x, uint8_t y) {
  terminal_put_cell(cx, cy, cell_under_cursor);
  cx = x;
  cy = y;
  cell_under_cursor = terminal_get_cell(cx, cy);
  if(cursor_visible)
    terminal_put_cell(cx, cy, cursor_cell);
}

void terminal_set_cursor_char(char c) {
  cursor_cell.c = c;
}

void terminal_set_cursor_color(enum vga_color bg, enum vga_color fg) {
  if(bg == VGA_DEFAULT) bg = VGA_BG_DEFAULT;
  if(fg == VGA_DEFAULT) fg = VGA_FG_DEFAULT;
  cursor_cell.bg = bg;
  cursor_cell.fg = fg;
}

void terminal_set_cursor_visibility(bool visible) {
  cursor_visible = visible;
}

uint8_t terminal_get_cursor_x(void) {
  return cx;
}

uint8_t terminal_get_cursor_y(void) {
  return cy;
}

char terminal_get_cursor_char(void) {
  return cursor_cell.c;
}

enum vga_color terminal_get_cursor_bg_color(void) { 
  return cursor_cell.bg;
}

enum vga_color terminal_get_cursor_fg_color(void) { 
  return cursor_cell.fg;
}

bool terminal_get_cursor_visibility(void) {
  return cursor_visible;
}
