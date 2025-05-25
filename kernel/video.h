#include <stdint.h>

#ifndef H_VIDOE
#define H_VIDOE

#define TERM_WIDTH 80
#define TERM_HEIGHT 25

enum video_type {
  VIDOE_TYPE_NONE = 0x00,
  VIDOE_TYPE_COLOR = 0x20,
  VIDOE_TYPE_MONOCHROME = 0x30,
};

enum vga_color : uint8_t {
  VGA_BLACK,
  VGA_BLUE,
  VGA_GREEN,
  VGA_CYAN,
  VGA_RED,
  VGA_MAGENTA,
  VGA_BROWN,
  VGA_LIGHT_GRAY,
  VGA_DARK_GRAY,
  VGA_LIGHT_BLUE,
  VGA_LIGHT_GREEN,
  VGA_LIGHT_CYAN,
  VGA_LIGHT_RED,
  VGA_LIGHT_MAGENTA,
  VGA_YELLOW,
  VGA_WHITE,
  VGA_DEFAULT,
}

struct vga_cell {
  char c;
  enum vga_color bg;
  enum vga_color fg;
}

struct video_type get_video_type(void);

void terminal_init(void);

void terminal_putcell(uint8_t x, uint8_t y, struct vga_cell cell);

void terminal_writechar(char c);
void terminal_write(const char *string);

void terminal_set_color(enum vga_color bg, enum vga_color fg);

void terminal_clear_screen(enum vga_color bg, enum vga_color fg); 

void terminal_move_cursor(uint8_t x, uint8_t y);
int terminal_get_cursor_x(void);
int terminal_get_cursor_y(void);
void terminal_set_cursor_color(enum vga_color bg, enum vga_color fg);
void terminal_set_cursor_visibility(bool visible);

struct vga_cell terminal_get_cell(uint8_t x, uint8_t y);

#endif
