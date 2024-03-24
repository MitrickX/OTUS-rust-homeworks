use std::cell::RefCell;
use std::rc::Rc;

pub type NodeRef<T> = Rc<RefCell<Option<Node<T>>>>;

pub struct Node<T> {
    value: T,
    next: NodeRef<T>,
}

pub struct List<T> {
    head: NodeRef<T>,
    tail: NodeRef<T>,
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        let head = Rc::new(RefCell::new(None));
        let tail = head.clone();
        List { head, tail }
    }

    pub fn push_back(&mut self, value: T) {
        let new_node = Node {
            value,
            next: Rc::new(RefCell::new(None)),
        };
        let new_node_ref = Rc::new(RefCell::new(Some(new_node)));

        if let Some(ref mut node) = *self.tail.borrow_mut() {
            node.next = new_node_ref.clone();
        }

        self.tail = new_node_ref.clone();

        if self.head.borrow().is_none() {
            self.head = self.tail.clone();
        }
    }

    pub fn push_front(&mut self, value: T) {
        if self.head.borrow().is_none() {
            self.push_back(value);
            return;
        }

        let new_node = Node {
            value,
            next: self.head.clone(),
        };
        let new_node_ref = Rc::new(RefCell::new(Some(new_node)));

        self.head = new_node_ref.clone();
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn push_back_words() {
        let mut result = List::<i32>::new();
        result.push_back(1);

        assert_eq!(1, result.head.borrow().as_ref().unwrap().value);
        assert_eq!(1, result.tail.borrow().as_ref().unwrap().value);

        result.push_back(2);

        assert_eq!(1, result.head.borrow().as_ref().unwrap().value);
        assert_eq!(2, result.tail.borrow().as_ref().unwrap().value);

        result.push_back(3);

        assert_eq!(1, result.head.borrow().as_ref().unwrap().value);
        assert_eq!(3, result.tail.borrow().as_ref().unwrap().value);
    }

    #[test]
    fn push_front_words() {
        let mut result = List::<i32>::new();
        result.push_front(1);

        assert_eq!(1, result.head.borrow().as_ref().unwrap().value);
        assert_eq!(1, result.tail.borrow().as_ref().unwrap().value);

        result.push_front(2);

        assert_eq!(2, result.head.borrow().as_ref().unwrap().value);
        assert_eq!(1, result.tail.borrow().as_ref().unwrap().value);

        result.push_front(3);

        assert_eq!(3, result.head.borrow().as_ref().unwrap().value);
        assert_eq!(1, result.tail.borrow().as_ref().unwrap().value);
    }
}