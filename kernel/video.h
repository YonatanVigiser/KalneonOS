#include <stdint.h>
#include <stdbool.h>

#ifndef H_VIDEO
#define H_VIDEo

#define TERM_WIDTH 80
#define TERM_HEIGHT 25

#define TAB_WIDTH 4

enum video_type {
  VIDOE_TYPE_NONE = 0x00,
  VIDOE_TYPE_COLOR = 0x20,
  VIDOE_TYPE_MONOCHROME = 0x30,
};

enum vga_color {
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
};

struct __attribute__((packed)) vga_cell {
  char c;
  uint8_t bg;
  uint8_t fg;
};

enum video_type terminal_get_video_type(void);

void terminal_init(void);

void terminal_put_cell(uint8_t x, uint8_t y, struct vga_cell cell);

struct vga_cell terminal_get_cell(uint8_t x, uint8_t y);

void terminal_write_char(char c);
void terminal_write(const char *string);

void terminal_set_color(enum vga_color bg, enum vga_color fg);
enum vga_color terminal_get_bg_color(void);
enum vga_color terminal_get_fg_color(void);

void terminal_clear_screen(enum vga_color bg, enum vga_color fg); 

void terminal_move_cursor(uint8_t x, uint8_t y);
void terminal_update_cursor(void);
void terminal_set_cursor_char(char c);
void terminal_set_cursor_color(enum vga_color bg, enum vga_color fg);
void terminal_set_cursor_visibility(bool visible);

uint8_t terminal_get_cursor_x(void);
uint8_t terminal_get_cursor_y(void);
char terminal_get_cursor_char(void);
enum vga_color terminal_get_cursor_bg_color(void);
enum vga_color terminal_get_cursor_fg_color(void);
bool terminal_get_cursor_visibility(void);

#endif
