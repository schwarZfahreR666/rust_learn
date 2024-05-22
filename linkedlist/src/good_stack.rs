
pub struct List<T> {
    head: Link<T>,
}

// 类型别名，type alias
type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

//每个集合类型应该实现 3 种迭代器类型：

//IntoIter - T  直接返回所有权
pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // access fields of a tuple struct numerically
        self.0.pop()
    }
}
//IterMut - &mut T  返回可变引用
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut { next: self.head.as_deref_mut() }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}

//Iter - &T  返回不可变引用
//这里的基本逻辑是我们持有一个当前节点的指针，当生成一个值后，该指针将指向下一个节点。
//iter中的next至少要比iter活的更长（源比派生要活得长, 属性比结构体要活得长）
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    // pub fn iter<'a>(&'a self) -> Iter<'a, T> {
    // 生命消除规则，只有一个参数或有&self时，输出生命周期自动标注
    pub fn iter(&self) -> Iter<T> {
        // 这里我们为 `iter` 声明一个生命周期 'a , 此时 `&self` 需要至少和 `Iter` 活得一样久
        //self.head.as_ref().map(|node| &**node)
        //self.head.as_ref().map::<&Node<T>, _>(|node| &node)
        // as_deref() 从 Option<T> (或 &Option<T>) 转换为 Option<&T::Target>
        Iter { next: self.head.as_deref() }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem: elem,
            //take方法可以拿到option的所有权，从选项中取出值，将 None 留在其位置。
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        //这里也需要拿到所有权
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        //self作为参数会传递所有权，如此就不能仅返回T的引用了,因为self会被释放
        //map作用在self.head中会拿到所有权，离开作用域会被释放，不能返回本地变量的引用
        //让 map 作用在引用上，而不是直接作用在 self.head 上
        //as_ref()方法将一个 Option<T> 变成了 Option<&T>
        //&Box<Node<T>>可以自动解引用
        self.head.as_ref().map(|node| {
            &node.elem
        })
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| {
            &mut node.elem
        })
    }
}

impl<T> Drop for List<T> {
    //不实现该方法会爆栈，因为Box的drop不是尾递归的
    fn drop(&mut self) {
        //该种实现直接操作Box智能指针
        let mut cur_link = self.head.take();

        while let Some(mut boxed_node) = cur_link {
            //cur_link拿到所有权后，超出作用范围自动drop
            cur_link = boxed_node.next.take();
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
fn peek() {
    let mut list = List::new();
    assert_eq!(list.peek(), None);
    assert_eq!(list.peek_mut(), None);
    list.push(1); list.push(2); list.push(3);

    assert_eq!(list.peek(), Some(&3));
    assert_eq!(list.peek_mut(), Some(&mut 3));

    list.peek_mut().map(|value| {
        *value = 42
    });

    assert_eq!(list.peek(), Some(&42));
    assert_eq!(list.pop(), Some(42));
}

#[test]
fn into_iter() {
    let mut list = List::new();
    list.push(1); list.push(2); list.push(3);

    let mut iter = list.into_iter();
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_mut() {
    let mut list = List::new();
    list.push(1); list.push(2); list.push(3);

    let mut iter = list.iter_mut();
    assert_eq!(iter.next(), Some(&mut 3));
    assert_eq!(iter.next(), Some(&mut 2));
    assert_eq!(iter.next(), Some(&mut 1));
}


#[test]
fn iter() {
    let mut list = List::new();
    list.push(1); list.push(2); list.push(3);

    let mut iter = list.iter();
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&1));
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

