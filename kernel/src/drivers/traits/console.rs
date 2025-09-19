pub trait Console: core::fmt::Write {
    fn init() -> Self;
    fn clear(&mut self);
    fn get_cursor_pos(&self) -> (u8, u8);
}
