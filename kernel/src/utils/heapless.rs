pub mod linked_list {
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
    size: u32,
  }

  impl LinkedList<'a, T> {
    pub fn new() -> Self {
      Self {
        first: None,
        size: 0,
      }
    }

    pub fn insert(&'a self, index: u32, value: T) {
      match self.first {
        Some(first_node) => {
          if index >= self.size {
            first_node.iter().last().next = Some(Node::new(value, None));
          }
          else if index == 0 {
            first_node = Some(Node::new(value, first_node.next()));
          } else {
            first_node.iter().nth(index - 1).next = Some(Node::new(value, first_node.iter().nth(index)));
          }
        },
        None => self.first = Some(Node::new(value)),
      };
      self.size += 1;
    }

    pub fn get(&'a self, index: u32) -> Result<T, ()> {
      if index >= self.size {
        return Err(());
      }
      Ok(self.first.iter().nth(index).value)
    }

    pub fn remove(&'a self, index: u32) -> Result<(), ()> {
      match self.first {
        Some(first_node) => {
          if index >= self.size {
            first_node.iter().next = Some(Node::new(value, None));
          }
          else if index == 0 {
            self.first = Some(Node::new(value, first_node));
          } else {
            first_node.iter().nth(index - 1).next = Some(Node::new(value, first_node.iter().nth(index + 1)));
          }
          self.size -= 1;
        },
        None => Err(());
      }
    }

    pub fn iter(&'a self) -> Iter<'a, T> {
      self.first.iter()
    }

    pub fn size(&'a self) -> u32 {
      self.size
    }
  }
}
