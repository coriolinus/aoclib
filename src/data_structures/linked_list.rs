//! Implementation of a doubly-linked list with cursor.

// Drawn from my implementation of the exercism exercise:
// https://github.com/coriolinus/exercism-rs/blob/master/doubly-linked-list/src/lib.rs

use std::fmt;
use std::ptr::NonNull;
type NodePtr<T> = Option<NonNull<Node<T>>>;

#[derive(Debug)]
struct Node<T> {
    item: T,
    /// next steps toward the back of the list
    next: NodePtr<T>,
    /// prev steps toward the front of the list
    prev: NodePtr<T>,
}

impl<T> Node<T> {
    fn new(item: T) -> Node<T> {
        Node {
            item,
            next: None,
            prev: None,
        }
    }

    fn into_ptr(self) -> NodePtr<T> {
        let heaped = Box::new(self);
        let ptr = Box::into_raw(heaped);
        debug_assert!(!ptr.is_null());
        NonNull::new(ptr)
    }

    /// Create an `Option<&Node<T>>` from a `NodePtr<T>`.
    ///
    /// Unsafe beause it has an arbitrary lifetime, which is inappropriate.
    /// For example, this can create a static lifetime.
    ///
    /// Use caution when returning references created by this function to
    /// ensure that they are forced into appropriate lifetimes.
    unsafe fn from_ptr<'a>(ptr: NodePtr<T>) -> Option<&'a Node<T>> {
        ptr.map(|ptr| ptr.as_ref())
    }

    /// Create an `Option<&mut Node<T>>` from a `NodePtr<T>`.
    ///
    /// Unsafe beause it has an arbitrary lifetime, which is inappropriate.
    /// For example, this can create a static lifetime.
    ///
    /// Use caution when returning references created by this function to
    /// ensure that they are forced into appropriate lifetimes.
    unsafe fn from_ptr_mut<'a>(ptr: NodePtr<T>) -> Option<&'a mut Node<T>> {
        ptr.map(|mut ptr| ptr.as_mut())
    }

    /// Create an `Option<Node<T>>` from a `NodePtr<T>`.
    ///
    /// Consumes `ptr`: it is always `None` after this function.
    fn owned_from_ptr(ptr: &mut NodePtr<T>) -> Option<Node<T>> {
        ptr.take()
            .map(|ptr| unsafe { *Box::from_raw(ptr.as_ptr()) })
    }

    fn len(&self) -> usize {
        1 + self.next.map_or(0, |next| unsafe { next.as_ref().len() })
    }
}

/// A doubly-linked list.
///
/// [`Vec<T>`] or [`VecDeque<T>`][std::collections::VecDeque] are almost always better choices.
/// However, there are occasions when fast insertion/removal in the middle of the list are
/// essential. In those cases, this data structure may be appropriate.
pub struct LinkedList<T> {
    front: NodePtr<T>,
    back: NodePtr<T>,
    len: usize,
}

impl<T> LinkedList<T> {
    /// Create a new linked list with no items.
    ///
    /// Performs no allocations.
    pub fn new() -> Self {
        LinkedList {
            front: None,
            back: None,
            len: 0,
        }
    }

    /// Count the items in this list.
    fn count(&self) -> usize {
        self.front.map_or(0, |node| unsafe { node.as_ref().len() })
    }

    /// Get the number of items in this list.
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.len, self.count());
        self.len
    }

    /// `true` when this list contains no items.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return a cursor positioned on the front element
    pub fn cursor_front(&mut self) -> Cursor<T> {
        Cursor::new(self, self.front)
    }

    /// Return a cursor positioned on the back element
    pub fn cursor_back(&mut self) -> Cursor<T> {
        Cursor::new(self, self.back)
    }

    /// Return an iterator that moves from front to back
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self, self.front)
    }

    /// Push an element to the back of the list.
    pub fn push_back(&mut self, element: T) {
        self.cursor_back().insert_after(element);
    }

    /// Push an element to the front of the list.
    pub fn push_front(&mut self, element: T) {
        self.cursor_front().insert_before(element);
    }

    /// Pop an element from the back of the list.
    pub fn pop_back(&mut self) -> Option<T> {
        self.cursor_back().take()
    }

    /// Pop an element from the front of the list.
    pub fn pop_front(&mut self) -> Option<T> {
        self.cursor_front().take()
    }

    /// Convert a node pointer to an element reference.
    fn elem(&self, ptr: NodePtr<T>) -> Option<&T> {
        unsafe { Node::from_ptr(ptr) }.map(|node| &node.item)
    }

    /// Convert a node pointer to a mutable element reference.
    fn elem_mut(&self, ptr: NodePtr<T>) -> Option<&mut T> {
        unsafe { Node::from_ptr_mut(ptr) }.map(|node| &mut node.item)
    }

    /// Get a reference to the front element in the list.
    pub fn front(&self) -> Option<&T> {
        self.elem(self.front)
    }

    /// Get a reference to the back element in the list.
    pub fn back(&self) -> Option<&T> {
        self.elem(self.back)
    }
}

impl<T: fmt::Debug> fmt::Debug for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: fmt::Display> fmt::Display for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut first = true;
        write!(f, "[")?;
        for item in self.iter() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        // basically the same as in the stdlib implementation of drop
        while self.pop_front().is_some() {}
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Send is safe: just moving the list between threads breaks no invariants;
// the pointers are still all valid.
unsafe impl<T: Send> Send for LinkedList<T> {}

// I'm _pretty sure_ Sync is safe: in the event of a mutable ref, only the ref
// can mutate the struct via the provided API, which is safe. In the event of
// many immutable refs, none of them mutate it anyway, which is safe.
//
// I do wish I were more confident of this, though.
unsafe impl<T: Sync> Sync for LinkedList<T> {}

#[derive(Debug)]
pub struct Cursor<'a, T> {
    ll: &'a mut LinkedList<T>,
    ptr: NodePtr<T>,
}

// the cursor is expected to act as if it is at the position of an element
// and it also has to work with and be able to insert into an empty list.
impl<T> Cursor<'_, T> {
    fn new(ll: &mut LinkedList<T>, ptr: NodePtr<T>) -> Cursor<T> {
        Cursor { ll, ptr }
    }

    /// Get a reference to the current element.
    pub fn elem(&self) -> Option<&T> {
        self.ll.elem(self.ptr)
    }

    /// Get a mutable reference to the current element.
    pub fn elem_mut(&mut self) -> Option<&mut T> {
        self.ll.elem_mut(self.ptr)
    }

    /// Move one position forward (towards the end) and
    /// return a reference to the new position.
    pub fn advance(&mut self) -> Option<&mut T> {
        self.ptr = unsafe { Node::from_ptr(self.ptr) }.and_then(|node| node.next);
        self.elem_mut()
    }

    /// Move one position backward (towards the start) and
    /// return a reference to the new position.
    pub fn retreat(&mut self) -> Option<&mut T> {
        self.ptr = unsafe { Node::from_ptr(self.ptr) }.and_then(|node| node.prev);
        self.elem_mut()
    }

    /// Examine the element one position forward (towards the end).
    pub fn peek_next(&self) -> Option<&T> {
        unsafe {
            Node::from_ptr(self.ptr)
                .and_then(|node| Node::from_ptr(node.next))
                .map(|next_node| &next_node.item)
        }
    }

    /// Examine the element one position back (towards the start).
    pub fn peek_prev(&self) -> Option<&T> {
        unsafe {
            Node::from_ptr(self.ptr)
                .and_then(|node| Node::from_ptr(node.prev))
                .map(|prev_node| &prev_node.item)
        }
    }

    /// Remove and return the element at the current position and move the cursor
    /// to the neighboring element that's closest to the back. This can be
    /// either the next or previous position.
    pub fn take(&mut self) -> Option<T> {
        Node::owned_from_ptr(&mut self.ptr).map(|node| {
            // update self ptr
            self.ptr = node.next.or(node.prev);

            // update external pointers
            if let Some(next) = unsafe { Node::from_ptr_mut(node.next) } {
                next.prev = node.prev;
            } else {
                self.ll.back = node.prev;
            }

            if let Some(prev) = unsafe { Node::from_ptr_mut(node.prev) } {
                prev.next = node.next;
            } else {
                self.ll.front = node.next;
            }

            self.ll.len -= 1;

            // return the item
            node.item
        })
    }

    /// Insert an element after the current position.
    pub fn insert_after(&mut self, element: T) {
        let new_node_ptr = Node::new(element).into_ptr();
        debug_assert!(new_node_ptr.is_some());
        self.ptr = match self.ptr {
            None => {
                self.ll.front = new_node_ptr;
                self.ll.back = new_node_ptr;
                new_node_ptr
            }
            Some(cur_ptr) => {
                unsafe {
                    let cur_node = cur_ptr.as_ptr();
                    // update both node pointers
                    (*new_node_ptr.unwrap().as_ptr()).prev = Some(cur_ptr);
                    (*new_node_ptr.unwrap().as_ptr()).next = (*cur_node).next;
                    // update external pointers
                    if let Some(next) = (*cur_node).next {
                        (*next.as_ptr()).prev = new_node_ptr;
                    } else {
                        self.ll.back = new_node_ptr;
                    }
                    // update self pointer
                    (*cur_node).next = new_node_ptr;
                }

                Some(cur_ptr)
            }
        };
        debug_assert!(self.ll.front.is_some());
        debug_assert!(self.ll.back.is_some());
        debug_assert!(self.ptr.is_some());
        self.ll.len += 1;
    }

    pub fn insert_before(&mut self, element: T) {
        let new_node_ptr = Node::new(element).into_ptr();
        debug_assert!(new_node_ptr.is_some());
        self.ptr = match self.ptr {
            None => {
                self.ll.front = new_node_ptr;
                self.ll.back = new_node_ptr;
                new_node_ptr
            }
            Some(cur_ptr) => {
                unsafe {
                    let cur_node = cur_ptr.as_ptr();
                    // update both node pointers
                    (*new_node_ptr.unwrap().as_ptr()).next = Some(cur_ptr);
                    (*new_node_ptr.unwrap().as_ptr()).prev = (*cur_node).prev;
                    // update external pointers
                    if let Some(prev) = (*cur_node).prev {
                        (*prev.as_ptr()).next = new_node_ptr;
                    } else {
                        self.ll.front = new_node_ptr;
                    }
                    // update self pointer
                    (*cur_node).prev = new_node_ptr;
                }

                Some(cur_ptr)
            }
        };
        debug_assert!(self.ll.front.is_some());
        debug_assert!(self.ll.back.is_some());
        debug_assert!(self.ptr.is_some());
        self.ll.len += 1;
    }

    /// Advance the cursor `n` items forward, returning `true` when it is still on an item.
    pub fn seek_forward(&mut self, n: usize) -> bool {
        (0..n).all(|_| self.advance().is_some())
    }

    /// Advance the cursor `n` items backward, returning `true` when it is still on an item.
    pub fn seek_backward(&mut self, n: usize) -> bool {
        (0..n).all(|_| self.retreat().is_some())
    }
}

impl<T: fmt::Display> fmt::Display for Cursor<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.ll.fmt(f)
    }
}

pub struct Iter<'a, T> {
    ll: &'a LinkedList<T>,
    ptr: NodePtr<T>,
}

impl<'a, T> Iter<'a, T> {
    fn new(ll: &'a LinkedList<T>, ptr: NodePtr<T>) -> Iter<'a, T> {
        Iter { ll, ptr }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let item = self.ll.elem(self.ptr);
        self.ptr = unsafe { Node::from_ptr(self.ptr) }.and_then(|node| node.next);
        item
    }
}

impl<T> std::iter::FromIterator<T> for LinkedList<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut list = Self::new();
        for elem in iter {
            list.push_back(elem);
        }
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_generic() {
        struct Foo;
        LinkedList::<Foo>::new();
    }

    #[test]
    fn basics_empty_list() {
        let list: LinkedList<i32> = LinkedList::new();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn basics_single_element_back() {
        let mut list: LinkedList<i32> = LinkedList::new();
        list.push_back(5);

        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());

        assert_eq!(list.pop_back(), Some(5));

        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn basics_push_pop_at_back() {
        let mut list: LinkedList<i32> = LinkedList::new();
        for i in 0..10 {
            list.push_back(i);
            assert_eq!(list.len(), i as usize + 1);
            assert!(!list.is_empty());
        }
        for i in (0..10).rev() {
            assert_eq!(list.len(), i as usize + 1);
            assert!(!list.is_empty());
            assert_eq!(i, list.pop_back().unwrap());
        }
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn basics_single_element_front() {
        let mut list: LinkedList<i32> = LinkedList::new();
        list.push_front(5);

        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());

        assert_eq!(list.pop_front(), Some(5));

        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn basics_push_pop_at_front() {
        let mut list: LinkedList<i32> = LinkedList::new();
        for i in 0..10 {
            list.push_front(i);
            assert_eq!(list.len(), i as usize + 1);
            assert!(!list.is_empty());
        }
        for i in (0..10).rev() {
            assert_eq!(list.len(), i as usize + 1);
            assert!(!list.is_empty());
            assert_eq!(i, list.pop_front().unwrap());
        }
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn basics_push_front_pop_back() {
        let mut list: LinkedList<i32> = LinkedList::new();
        for i in 0..10 {
            list.push_front(i);
            assert_eq!(list.len(), i as usize + 1);
            assert!(!list.is_empty());
        }
        for i in 0..10 {
            assert_eq!(list.len(), 10 - i as usize);
            assert!(!list.is_empty());
            assert_eq!(i, list.pop_back().unwrap());
        }
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn basics_push_back_pop_front() {
        let mut list: LinkedList<i32> = LinkedList::new();
        for i in 0..10 {
            list.push_back(i);
            assert_eq!(list.len(), i as usize + 1);
            assert!(!list.is_empty());
        }
        for i in 0..10 {
            assert_eq!(list.len(), 10 - i as usize);
            assert!(!list.is_empty());
            assert_eq!(i, list.pop_front().unwrap());
        }
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn iter() {
        let mut list: LinkedList<i32> = LinkedList::new();
        for num in 0..10 {
            list.push_back(num);
        }

        for (num, &entered_num) in (0..10).zip(list.iter()) {
            assert_eq!(num, entered_num);
        }
    }

    #[test]
    fn cursor_insert_before_on_empty_list() {
        // insert_after on empty list is already tested via push_back()
        let mut list = LinkedList::new();
        list.cursor_front().insert_before(0);
        assert_eq!(Some(0), list.pop_front());
    }

    #[test]
    fn cursor_insert_after_in_middle() {
        let mut list = (0..10).collect::<LinkedList<_>>();

        {
            let mut cursor = list.cursor_front();
            let didnt_run_into_end = cursor.seek_forward(4);
            assert!(didnt_run_into_end);

            for n in (0..10).rev() {
                cursor.insert_after(n);
            }
        }

        assert_eq!(list.len(), 20);

        let expected = (0..5).chain(0..10).chain(5..10);

        assert!(expected.eq(list.iter().cloned()));
    }

    #[test]
    fn cursor_insert_before_in_middle() {
        let mut list = (0..10).collect::<LinkedList<_>>();

        {
            let mut cursor = list.cursor_back();
            let didnt_run_into_end = cursor.seek_backward(4);
            assert!(didnt_run_into_end);

            for n in 0..10 {
                cursor.insert_before(n);
            }
        }

        assert_eq!(list.len(), 20);

        let expected = (0..5).chain(0..10).chain(5..10);

        assert!(expected.eq(list.iter().cloned()));
    }

    // "iterates" via next() and checks that it visits the right elements
    #[test]
    fn cursor_next_and_peek() {
        let mut list = (0..10).collect::<LinkedList<_>>();
        let mut cursor = list.cursor_front();

        assert_eq!(cursor.elem_mut(), Some(&mut 0));

        for n in 1..10 {
            let next = cursor.advance().cloned();
            assert_eq!(next, Some(n));
            assert_eq!(next, cursor.elem_mut().cloned());
        }
    }

    // "iterates" via prev() and checks that it visits the right elements
    #[test]
    fn cursor_prev_and_peek() {
        let mut list = (0..10).collect::<LinkedList<_>>();
        let mut cursor = list.cursor_back();

        assert_eq!(cursor.elem_mut(), Some(&mut 9));

        for n in (0..9).rev() {
            let prev = cursor.retreat().cloned();
            assert_eq!(prev, Some(n));
            assert_eq!(prev, cursor.elem_mut().cloned());
        }
    }

    // removes all elements starting from the middle
    #[test]
    fn cursor_take() {
        let mut list = (0..10).collect::<LinkedList<_>>();
        let mut cursor = list.cursor_front();
        cursor.seek_forward(5);

        for expected in (5..10).chain((0..5).rev()) {
            assert_eq!(cursor.take(), Some(expected));
        }
    }

    // checks number of drops
    // may pass for incorrect programs if double frees happen
    // exactly as often as destructor leaks
    #[test]
    fn drop_no_double_frees() {
        use std::cell::Cell;
        struct DropCounter<'a>(&'a Cell<usize>);

        impl<'a> Drop for DropCounter<'a> {
            fn drop(&mut self) {
                let num = self.0.get();
                self.0.set(num + 1);
            }
        }

        const N: usize = 15;

        let counter = Cell::new(0);
        let list = std::iter::repeat_with(|| DropCounter(&counter))
            .take(N)
            .collect::<LinkedList<_>>();

        assert_eq!(list.len(), N);
        drop(list);
        assert_eq!(counter.get(), N);
    }

    #[test]
    fn drop_large_list() {
        drop((0..2_000_000).collect::<LinkedList<i32>>());
    }

    // These are compile time tests. They won't compile unless your
    // code passes.
    #[test]
    fn advanced_linked_list_is_send_sync() {
        trait AssertSend: Send {}
        trait AssertSync: Sync {}

        impl<T: Send> AssertSend for LinkedList<T> {}
        impl<T: Sync> AssertSync for LinkedList<T> {}
    }

    #[allow(dead_code)]
    #[test]
    fn advanced_is_covariant() {
        fn a<'a>(x: LinkedList<&'static str>) -> LinkedList<&'a str> {
            x
        }

        fn a_iter<'a>(i: Iter<'static, &'static str>) -> Iter<'a, &'a str> {
            i
        }
    }
}
