use alloc::boxed::Box;
use core::ptr::null_mut;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::{Relaxed, Release};

pub trait IntrusiveNode
where
    Self: Sized,
{
    fn next(&self) -> &AtomicPtr<Self>;

    fn previous(&self) -> &AtomicPtr<Self>;
}

pub trait BoxedIntrusiveNode
where
    Self: Sized,
{
    fn into_mut_ptr(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    fn into_boxed(self: *mut Self) -> Box<Self> {
        unsafe { Box::from_raw(self) }
    }
}

impl<T> BoxedIntrusiveNode for T
where
    T: IntrusiveNode,
{}

pub struct LockFreeIntrusiveLinkedList<T: IntrusiveNode> {
    head: AtomicPtr<T>,
    tail: AtomicPtr<T>,
}

impl<T: IntrusiveNode> LockFreeIntrusiveLinkedList<T> {
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(null_mut()),
            tail: AtomicPtr::new(null_mut()),
        }
    }

    /// Atomically inserts the given node at the front of the list.
    /// The node must not point to another node.
    ///
    /// Returns `Ok(())` if the node was successfully inserted, `Err(())` otherwise.
    pub fn push_front(&self, t: *mut T) -> Result<(), ()> {
        assert_eq!(null_mut(), unsafe { (*t).previous() }.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*t).next() }.load(Relaxed));

        loop {
            let old_head = self.head.load(Relaxed);
            let list_empty = old_head.is_null();
            unsafe { (*t).next() }.store(old_head, Relaxed);

            if self
                .head
                .compare_exchange(old_head, t, Release, Relaxed)
                .is_ok()
            {
                if list_empty {
                    // If the list was empty, we need to update the tail pointer.
                    // If the tail changed in the meantime, that means that someone else
                    // added something to the list, and we trust them to have updated
                    // the tail pointer.
                    self.tail.compare_exchange(null_mut(), t, Release, Relaxed).ok();
                } else {
                    // If we had a non-empty list, we need to update the previous pointer
                    // of the old head element.
                    unsafe { (*old_head).previous() }.compare_exchange(null_mut(), t, Release, Relaxed).ok();
                }
                return Ok(());
            }
        }
    }

    pub fn push_back(&self, t: *mut T) -> Result<(), ()> {
        assert_eq!(null_mut(), unsafe { (*t).previous() }.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*t).next() }.load(Relaxed));

        loop {
            let old_tail = self.tail.load(Relaxed);
            let list_empty = old_tail.is_null();
            unsafe { (*t).previous() }.store(old_tail, Relaxed);

            if self
                .tail
                .compare_exchange(old_tail, t, Release, Relaxed)
                .is_ok()
            {
                if list_empty {
                    // If the list was empty, we need to update the head pointer.
                    // If the head changed in the meantime, that means that someone else
                    // added something to the list, and we trust them to have updated
                    // the head pointer.
                    self.head.compare_exchange(null_mut(), t, Release, Relaxed).ok();
                } else {
                    // If we had a non-empty list, we need to update the next pointer
                    // of the old tail element.
                    unsafe { (*old_tail).next() }.compare_exchange(null_mut(), t, Release, Relaxed).ok();
                }
                return Ok(());
            }
        }
    }

    pub fn pop_front(&self) -> Option<*mut T> {
        let mut old_head = self.head.load(Relaxed);
        loop {
            if old_head.is_null() {
                return None;
            }
            let new_head = unsafe { (*old_head).next().load(Relaxed) };
            let list_now_empty = new_head.is_null();
            match self.head.compare_exchange(old_head, new_head, Release, Relaxed) {
                Ok(v) => {
                    if list_now_empty {
                        // If the list is now empty, we need to update the tail pointer.
                        // We set that to null. However, if tail changed in the meantime,
                        // that means that someone else added something to the list, and we
                        // trust them to have updated the tail pointer.
                        self.tail.compare_exchange(v, null_mut(), Release, Relaxed).ok();
                    } else {
                        // If the list is not empty, we need to update the previous pointer
                        // of the new head element.
                        unsafe { (*new_head).previous() }.compare_exchange(old_head, null_mut(), Release, Relaxed).ok();
                    }
                    return Some(old_head);
                }
                Err(v) => old_head = v,
            }
        }
    }

    pub fn append(&self, other: &Self) {
        let other_head = other.head.swap(null_mut(), Relaxed);
        if other_head.is_null() {
            return;
        }

        // We took the full list content, so we are able to just walk to the tail
        // without considering concurrent changes.
        // This assumption holds as long as the previous and next pointers of elements
        // are not manually adapted externally.
        let mut other_tail = other_head;
        while !unsafe { (*other_tail).next() }.load(Relaxed).is_null() {
            other_tail = unsafe { (*other_tail).next() }.load(Relaxed);
        }

        // If the other list's tail changed after we "took" the whole content, that means
        // that someone else added something to the list, and we trust them to have updated
        // the tail pointer.
        other.tail.compare_exchange(other_tail, null_mut(), Release, Relaxed).ok();

        loop {
            let old_tail = self.tail.load(Relaxed);
            if old_tail.is_null() {
                if self.tail.compare_exchange(old_tail, other_tail, Release, Relaxed).is_ok() {
                    // If the tail was null, the head was null as well, so we need to update that.
                    // If the head changed, that means that someone else added something
                    // to the list, and we trust them to have updated the head pointer.
                    self.head.compare_exchange(null_mut(), other_head, Release, Relaxed).ok();
                    return;
                }
            } else {
                if unsafe { (*old_tail).next() }.compare_exchange(null_mut(), other_head, Release, Relaxed).is_ok() {
                    if self.tail.compare_exchange(old_tail, other_tail, Release, Relaxed).is_ok() {
                        unsafe { (*other_head).previous() }.compare_exchange(null_mut(), old_tail, Release, Relaxed)
                            .expect("something changed about the other list after we took it, this must not happen");
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(feature = "kernel_test")]
mod tests {
    use kernel_test_framework::kernel_test;

    use super::*;

    struct Node {
        next: AtomicPtr<Node>,
        previous: AtomicPtr<Node>,
        value: u32,
    }

    impl IntrusiveNode for Node {
        fn next(&self) -> &AtomicPtr<Self> {
            &self.next
        }

        fn previous(&self) -> &AtomicPtr<Self> {
            &self.previous
        }
    }

    #[kernel_test]
    fn test_append() {
        let node1 = Box::into_raw(Box::new(Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 0,
        }));
        let node2 = Box::into_raw(Box::new(Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 1,
        }));
        let node3 = Box::into_raw(Box::new(Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 2,
        }));
        let node4 = Box::into_raw(Box::new(Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 3,
        }));
        let node5 = Box::into_raw(Box::new(Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 4,
        }));
        let node6 = Box::into_raw(Box::new(Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 5,
        }));

        let list1 = LockFreeIntrusiveLinkedList::new();
        list1.push_front(node1).unwrap();
        list1.push_front(node2).unwrap();
        list1.push_front(node3).unwrap();

        let list2 = LockFreeIntrusiveLinkedList::new();
        list2.push_front(node4).unwrap();
        list2.push_front(node5).unwrap();
        list2.push_front(node6).unwrap();

        list1.append(&list2); // list1 order must now be 3, 2, 1, 6, 5, 4, list2 empty

        assert_eq!(null_mut(), list2.head.load(Relaxed));
        assert_eq!(null_mut(), list2.tail.load(Relaxed));

        assert_eq!(node3, list1.head.load(Relaxed));
        assert_eq!(node4, list1.tail.load(Relaxed));

        assert_eq!(node6, unsafe { (*node1).next().load(Relaxed) });
        assert_eq!(node1, unsafe { (*node6).previous().load(Relaxed) });
    }

    #[kernel_test]
    fn test_atomic_intrusive_linked_list() {
        let list = LockFreeIntrusiveLinkedList::new();
        let node1 = Box::into_raw(Box::new(Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 0,
        }));
        let node2 = Box::into_raw(Box::new(Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 1,
        }));
        let node3 = Box::into_raw(Box::new(Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 2,
        }));

        assert_eq!(null_mut(), list.head.load(Relaxed));
        assert_eq!(null_mut(), list.tail.load(Relaxed));

        list.push_front(node1).unwrap();
        assert_eq!(node1, list.head.load(Relaxed));
        assert_eq!(node1, list.tail.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).next().load(Relaxed) });

        list.push_front(node2).unwrap();
        assert_eq!(node2, list.head.load(Relaxed));
        assert_eq!(node1, list.tail.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*node2).previous().load(Relaxed) });
        assert_eq!(node1, unsafe { (*node2).next().load(Relaxed) });
        assert_eq!(node2, unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).next().load(Relaxed) });

        list.push_front(node3).unwrap();
        assert_eq!(node3, list.head.load(Relaxed));
        assert_eq!(node1, list.tail.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*node3).previous().load(Relaxed) });
        assert_eq!(node2, unsafe { (*node3).next().load(Relaxed) });
        assert_eq!(node3, unsafe { (*node2).previous().load(Relaxed) });
        assert_eq!(node1, unsafe { (*node2).next().load(Relaxed) });
        assert_eq!(node2, unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).next().load(Relaxed) });

        let mut current = list.head.load(Relaxed);
        let mut count = 0;
        while !current.is_null() {
            count += 1;
            assert_eq!(unsafe { (*current).value }, 3 - count);
            unsafe {
                current = (*current).next.load(Relaxed);
            }
        }
        assert_eq!(count, 3);

        let node = list.pop_front();
        assert_eq!(node, Some(node3));
        assert_eq!(node2, list.head.load(Relaxed));
        assert_eq!(node1, list.tail.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*node2).previous().load(Relaxed) });
        assert_eq!(node1, unsafe { (*node2).next().load(Relaxed) });
        assert_eq!(node2, unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).next().load(Relaxed) });

        let node = list.pop_front();
        assert_eq!(node, Some(node2));
        assert_eq!(node1, list.head.load(Relaxed));
        assert_eq!(node1, list.tail.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).next().load(Relaxed) });

        let node = list.pop_front();
        assert_eq!(node, Some(node1));
        assert_eq!(null_mut(), list.head.load(Relaxed));
        assert_eq!(null_mut(), list.tail.load(Relaxed));

        let node = list.pop_front();
        assert_eq!(node, None);
    }
}