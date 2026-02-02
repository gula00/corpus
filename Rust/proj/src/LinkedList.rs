use std::fmt;

struct LinkedList {
    head: Option<Box<Node>>,
    size: usize,
}

struct Node {
    // just forward
    value: u32,
    next: Option<Box<Node>>, // owned pointer to somewhere on the heap
}

impl Node {
    pub fn new(value: u32, next: Option<Box<Node>>) -> Node {
        Node {
            value: value,
            next: next,
        }
    }
}

impl LinkedList {
    pub fn new() -> LinkedList {
        LinkedList {
            head: None,
            size: 0,
        }
    }

    pub fn get_size(&self) -> usize {
        self.size // (*self)
    }

    pub fn is_empty(&self) -> bool {
        self.get_size() == 0
    }

    pub fn push(&mut self, value: u32) {
        let new_node: Box<Node> = Box::new(Node::new(value, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<u32> {
        let node: Box<Node> = self.head.take()?;
        // self.head = self.head.next;
        self.head = node.next;
        self.size -= 1;
        Some(node.value)
    }
}

impl fmt::Display for LinkedList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current: &Option<Box<Node>> = &self.head;
        let mut result = String::new();
        loop {
            match current {
                Some(node) => {
                    result = format!("{} {}", result, node.value);
                    current = &node.next;
                }
                None => break,
            }
        }
        write!(f, "{}", result)
    }
}

impl Drop for LinkedList {
    fn drop(&mut self) {
        println!("Here comes the drop.");
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

fn main() {
    let mut list: LinkedList = LinkedList::new();
    assert!(list.is_empty());
    println!("{}", list.is_empty());

    let mut x: Option<u32> = Some(5);
    let x_ref: &mut Option<u32> = &mut x;
    // println!("result of take: {:?}", x_ref.take());
    // println!("left at x: {:?}", x);
    for i in 1..10 {
        list.push(i);
    }
    println!("{}", list);
    println!("list size: {}", list.get_size());
    println!("top element: {}", list.pop().unwrap());
    println!("{}", list);
    println!("list size: {}", list.get_size());
}
