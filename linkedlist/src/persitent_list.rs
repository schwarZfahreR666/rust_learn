// in third.rs
use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn prepend(&self, elem: T) -> List<T> {
        List { head: Some(Rc::new(Node {
            elem: elem,
            //option也实现了clone trait，会clone内部元素返回option
            next: self.head.clone(),
        }))}
    }

    pub fn tail(&self) -> List<T> {
        //map:Maps an Option<T> to Option<U>，闭包返回值会自动被option包裹
        //and_then(flatmap):Returns None if the option is None,
        //otherwise calls f with the wrapped value and returns the result.
        //闭包要返回option
        //node.next包裹着option，所以使用and_then，使用map会返回option<option>
        // List { head: self.head.as_ref().map(|node| node.next.clone()) }
        List { head: self.head.as_ref().and_then(|node| node.next.clone()) }
    }

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem )
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter { next: self.head.as_deref() }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            //as_deref():从 Option<T> (或 &Option<T>) 转换为Option<&<T as Deref>::Target>
            //将原始 Option 保留在原位，创建一个带有对原始 Option 的引用的新 Option，并通过 Deref 强制执行其内容。
            //Rc实现了Deref trait，被解引用了
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            //Rc::try_unwrap会判断当前的 Rc 是否只有一个强引用，若是，则返回 Rc 持有的值，否则返回一个错误
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);

    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}
