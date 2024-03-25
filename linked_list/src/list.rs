use std::cell::RefCell;
use std::rc::Rc;

pub type NodeRef<T> = Rc<RefCell<Option<Node<T>>>>;

pub struct Node<T> {
    value: T,
    next: NodeRef<T>,
}

impl<T> Node<T> {
    fn new_none(value: T) -> Self {
        Node { value, next: Rc::new(RefCell::new(None)) }
    }

    fn new_some(value: T, next: NodeRef<T>) -> Self {
        Node { value, next }
    }

    fn new_none_ref(value: T) -> NodeRef<T> {
        Rc::new(RefCell::new(Some(Node::new_none(value))))
    }

    fn new_some_ref(value: T, next: NodeRef<T>) -> NodeRef<T> {
        Rc::new(RefCell::new(Some(Node::new_some(value, next))))
    }
}

pub struct List<T> {
    head: NodeRef<T>,
    tail: NodeRef<T>,
    size: usize,
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
        List { head, tail, size: 0 }
    }

    pub fn push_back(&mut self, value: T) {
        let new_node_ref = Node::new_none_ref(value);

        if let Some(ref mut node) = *self.tail.borrow_mut() {
            node.next = new_node_ref.clone();
        }

        self.tail = new_node_ref.clone();

        if self.head.borrow().is_none() {
            self.head = self.tail.clone();
        }

        self.size += 1;
    }

    pub fn push_front(&mut self, value: T) {
        if self.size == 0 {
            self.push_back(value);
            return;
        }

        let new_node_ref = Node::new_some_ref(value, self.head.clone());

        self.head = new_node_ref.clone();

        self.size += 1;
    }

    pub fn push_after(&mut self, idx: usize, value: T) {
        if self.size <= idx {
            self.push_back(value);
            return;
        }

        let mut p = self.head.clone();

        for _ in 0..idx {
            if let Some(ref node) = *p.clone().borrow() {
                p = node.next.clone();
            };
        }

        if let Some(ref mut node) = *p.borrow_mut() {
            let new_node_ref = Node::new_some_ref(value, node.next.clone());
            node.next = new_node_ref.clone();
        };

        self.size += 1;
    }

    pub fn set(&mut self, idx: usize, value: T) {
        if self.size <= idx {
            return;
        }

        let mut p = self.head.clone();

        for _ in 0..idx {
            if let Some(ref node) = *p.clone().borrow() {
                p = node.next.clone();
            };
        }

        if let Some(ref mut node) = *p.borrow_mut() {
            node.value = value;
        };
    }
}

pub struct ListIterator<T> {
    cur: NodeRef<T>,
}

impl <T: Copy> Iterator for ListIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref node) = *self.cur.clone().borrow() {
            let val = Some(node.value);
            self.cur = node.next.clone();
            val
        } else {
            None
        }
    }
}

impl <T: Copy> IntoIterator for &List<T> {
    type Item = T;
    type IntoIter = ListIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        ListIterator { cur: self.head.clone() }
    }
}

impl <T : Copy> List <T> {
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::<T>::new();

        for i in self {
            result.push(i)
        }

        result
    }

    pub fn split(&self, n: usize) -> (List<T>, List<T>) {
        let mut p = self.head.clone();

        let mut left = List::new();
        let mut right = List::new();
        
        for idx in 0..self.size {
            if let Some(ref node) = *p.clone().borrow() {
                if idx < n {
                    left.push_back(node.value);
                } else {
                    right.push_back(node.value);
                }
                p = node.next.clone();
            } else {
                break;
            }
        }

        (left, right)
    }
}

#[cfg(test)]
mod tests {
    use crate::list::List;

    #[test]
    fn push_back_works() {
        let mut result = List::<i32>::new();
        result.push_back(1);
        result.push_back(2);
        result.push_back(3);

        assert_eq!("[1, 2, 3]", format!("{:?}", result.to_vec()));
    }

    #[test]
    fn push_front_works() {
        let mut result = List::<i32>::new();
        result.push_front(1);
        result.push_front(2);
        result.push_front(3);

        assert_eq!("[3, 2, 1]", format!("{:?}", result.to_vec()));
    }

    #[test]
    fn push_after_works() {
        let mut result = List::<i32>::new();

        result.push_after(0, -1);

        result.push_back(1);
        result.push_back(2);
        result.push_back(3);
        result.push_front(0);

        assert_eq!("[0, -1, 1, 2, 3]", format!("{:?}", result.to_vec()));

        result.push_after(0, 100);
        result.push_after(0, 101);
        result.push_after(3, 200);
        result.push_after(3, 201);

        result.push_after(6, 300);
        result.push_after(6, 301);

        result.push_after(9, 400);
        result.push_after(9, 401);

        result.push_after(1000, 500);

        assert_eq!("[0, 101, 100, -1, 201, 200, 1, 301, 300, 2, 401, 400, 3, 500]", format!("{:?}", result.to_vec()));
    }

    #[test]
    fn split_works() {
        let empty = List::<i32>::new();
        let (left, right) = empty.split(0);
        assert_eq!("[]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let (left, right) = empty.split(1);
        assert_eq!("[]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let (left, right) = empty.split(100);
        assert_eq!("[]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let mut one_elem = List::<i32>::new();
        one_elem.push_back(1);
        let (left, right) = one_elem.split(0);
        assert_eq!("[]", format!("{:?}", left.to_vec()));
        assert_eq!("[1]", format!("{:?}", right.to_vec()));

        let (left, right) = one_elem.split(1);
        assert_eq!("[1]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let (left, right) = one_elem.split(2);
        assert_eq!("[1]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let (left, right) = one_elem.split(100);
        assert_eq!("[1]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let mut two_elems = List::<i32>::new();
        two_elems.push_back(1);
        two_elems.push_back(2);
        let (left, right) = two_elems.split(0);
        assert_eq!("[]", format!("{:?}", left.to_vec()));
        assert_eq!("[1, 2]", format!("{:?}", right.to_vec()));

        let (left, right) = two_elems.split(1);
        assert_eq!("[1]", format!("{:?}", left.to_vec()));
        assert_eq!("[2]", format!("{:?}", right.to_vec()));

        let (left, right) = two_elems.split(2);
        assert_eq!("[1, 2]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let (left, right) = two_elems.split(3);
        assert_eq!("[1, 2]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let (left, right) = two_elems.split(100);
        assert_eq!("[1, 2]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let mut three_elems = List::<i32>::new();
        three_elems.push_back(1);
        three_elems.push_back(2);
        three_elems.push_back(3);

        let (left, right) = three_elems.split(0);
        assert_eq!("[]", format!("{:?}", left.to_vec()));
        assert_eq!("[1, 2, 3]", format!("{:?}", right.to_vec()));

        let (left, right) = three_elems.split(1);
        assert_eq!("[1]", format!("{:?}", left.to_vec()));
        assert_eq!("[2, 3]", format!("{:?}", right.to_vec()));

        let (left, right) = three_elems.split(2);
        assert_eq!("[1, 2]", format!("{:?}", left.to_vec()));
        assert_eq!("[3]", format!("{:?}", right.to_vec()));

        let (left, right) = three_elems.split(3);
        assert_eq!("[1, 2, 3]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let (left, right) = three_elems.split(4);
        assert_eq!("[1, 2, 3]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

        let (left, right) = three_elems.split(100);
        assert_eq!("[1, 2, 3]", format!("{:?}", left.to_vec()));
        assert_eq!("[]", format!("{:?}", right.to_vec()));

    }

    #[test]
    fn iter_works() {
        let mut list = List::<i32>::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut vec = Vec::<i32>::new();
        for i in &list {
            vec.push(i);
        }

        assert_eq!("[1, 2, 3]", format!("{:?}", vec));
    }

    #[test]
    fn set_works() {
        let mut empty = List::<i32>::new();
        empty.set(0, 1);

        assert_eq!("[]", format!("{:?}", empty.to_vec()));

        empty.set(100, 1);

        assert_eq!("[]", format!("{:?}", empty.to_vec()));

        let mut list = List::<i32>::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        list.set(0, 100);
        list.set(1, 200);
        list.set(2, 300);
        list.set(3, 400);

        assert_eq!("[100, 200, 300]", format!("{:?}", list.to_vec()));
    }
}