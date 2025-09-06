pub struct Node<'a, T> {
    value: T,
    next: Option<&'a Node<'a, T>>,
}

impl<'a, T> Node<'a, T> {
    pub fn new(value: T, next: Option<&'a Self>) -> Self {
        Node { value, next }
    }

    pub fn iter(&'a self) -> Iter<'a, T> {
        Iter {
            current: Some(self),
        }
    }
}

struct Iter<'a, T> {
    current: Option<&'a Node<'a, T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        match self.current {
            Some(Node { value, next }) => {
                self.current = *next;
                Some(value)
            }
            None => None,
        }
    }
}

pub struct LinkedList<'a, T> {
    first: Option<Node<'a, T>>,
    size: usize,
}

impl<'a, T> LinkedList<'a, T> {
    pub fn new() -> Self {
        Self {
            first: None,
            size: 0,
        }
    }

    pub fn insert(&'a self, index: usize, value: T) {
        match self.first {
            Some(first_node) => {
                if index >= self.size - 1{
                    first_node.iter().last().unwrap().next = Some(Node::new(value, None));
                } else if index == 0 {
                    self.first = Some(Node::new(value, first_node));
                } else {
                    first_node.iter().nth(index - 1).unwrap().next =
                        Some(Node::new(value, first_node.iter().nth(index)));
                }
            }
            None => self.first = Some(Node::new(value, None)),
        };
        self.size += 1;
    }

    pub fn get(&'a self, index: usize) -> Option<T> {
        Some(self.first.iter().nth(index)?.value)
    }

    pub fn remove(&'a self, index: usize) -> Result<(), ()> {
        match self.first {
            Some(first_node) => {
                if index >= self.size - 1 {
                    first_node.iter().nth(self.size - 2).next = None;
                } else if index == 0 {
                    self.first = first_node.iter().next();
                } else {
                    first_node.iter().nth(index - 1).unwrap().next = first_node.iter().nth(index).next;
                }
                self.size -= 1;
                Ok(())
            }
            None => Err(()),
        }
    }

    pub fn iter(&'a self) -> Iter<'a, T> {
        self.first.iter()
    }

    pub fn size(&'a self) -> usize {
        self.size
    }
}
