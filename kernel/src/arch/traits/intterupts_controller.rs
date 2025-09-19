pub trait IntteruptsController {
    fn init() -> Self;

    fn enable();
}
