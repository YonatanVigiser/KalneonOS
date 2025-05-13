#include <stdint.h>

volitale uint8_t *video_buff;

enum video_type {
  NONE = 0x00,
  COLOR = 0x20,
  MONOCHROME = 0x30,
};

uint16_t detect_bios_area_hardware(void) {
  const uint16_t* bda_detected_hardware_ptr = (const uint16_t*) 0x410;
  return *bda_detected_hardware_ptr;
}

enum video_type get_video_type(void) {
  return (enum video_type) (detect_bios_area_hardware() & 0x30);
}

enum video_type init_display(void) {
  enum video_type = get_video_type();
  switch(video_type) {
    case (COLOR):
      video_type = (volitale uint8_t*) 0xB8000;
      break;
    case (MONOCHROME):
      video_type = (volitale uint8_t*) 0xB0000;
      break;
    default:
      video_type = (volitale uint8_t*) 0xB8000; // If not detected, use this as a fake buffer
  }
  return video_type;
}

void draw_pixel(uint8_t , uint8_t x, uint8_t y, 
