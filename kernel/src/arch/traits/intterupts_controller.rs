pub trait IntteruptsController {
    fn init() -> Self;

    fn enable_intterupts();
    fn disable_intterupts();

}
