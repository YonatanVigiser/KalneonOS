pub trait Indexable {
    fn as_index(&self) -> usize;
    fn from_index(index: usize) -> Self;
    fn next(&self) -> Self where Self: Sized {
        Self::from_index(self.as_index() + 1)
    }
    fn prev(&self) -> Self where Self: Sized {
        Self::from_index(self.as_index() - 1)
    }
    fn next_nth(&self, nth: usize) -> Self where Self: Sized {
        Self::from_index(self.as_index() + nth)
    }
    fn prev_nth(&self, nth: usize) -> Self where Self: Sized {
        Self::from_index(self.as_index() - nth)
    }
}
