use std::mem;

pub struct List {
    head: Link,
}

enum Link {
    Empty,
    //如果不放在堆上，则大小是动态的，编译报错
    More(Box<Node>),
}

struct Node {
    elem: String,
    next: Link,
}

impl List {
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    pub fn push(&mut self, elem: String) {
        let new_node = Box::new(Node {
            elem: elem,
            //replace方法把head的所有权置换出来赋值给next
            next: mem::replace(&mut self.head, Link::Empty),
        });

        self.head = Link::More(new_node);
    }

    pub fn pop(&mut self) -> Option<String> {
        //这里也需要拿到所有权
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}

impl Drop for List {
    //不实现该方法会爆栈，因为Box的drop不是尾递归的
    fn drop(&mut self) {
        //该种实现直接操作Box智能指针
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);

        while let Link::More(mut boxed_node) = cur_link {
            //cur_link拿到所有权后，超出作用范围自动drop
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
        }

        // //该种实现方法在element实现copy trait时需要拷贝每个node值，如果数据量过大性能会差
        // while let Some(_) = self.pop() {}
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push("1".to_string());
        list.push("2".to_string());
        list.push("3".to_string());

        // Check normal removal
        assert_eq!(list.pop(), Some("3".to_string()));
        assert_eq!(list.pop(), Some("2".to_string()));

        // Push some more just to make sure nothing's corrupted
        list.push("4".to_string());
        list.push("5".to_string());

        // Check normal removal
        assert_eq!(list.pop(), Some("5".to_string()));
        assert_eq!(list.pop(), Some("4".to_string()));

        // Check exhaustion
        assert_eq!(list.pop(), Some("1".to_string()));
        assert_eq!(list.pop(), None);
    }
}

#[test]
fn long_list() {
    let mut list = List::new();
    let mut s = "".to_string();
    for i in 0..50000{
        s += i.to_string().as_str();
    }
    for _i in 0..100000 {
        list.push(s.clone());
    }
    drop(list);
}