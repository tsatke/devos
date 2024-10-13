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

pub struct LockFreeIntrusiveLinkedList<T: IntrusiveNode> {
    head: AtomicPtr<T>,
    tail: AtomicPtr<T>,
}

macro_rules! next {
    ($t:expr) => {{
        let t = $t;
        if t.is_null() {
            panic!("can't get next because the node ptr is null")
        }
        unsafe { (*t).next() }
    }};
}

macro_rules! previous {
    ($t:expr) => {{
        let t = $t;
        if t.is_null() {
            panic!("can't get previous because the node ptr is null")
        }
        unsafe { (*t).previous() }
    }};
}

impl<T: IntrusiveNode> LockFreeIntrusiveLinkedList<T> {
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(null_mut()),
            tail: AtomicPtr::new(null_mut()),
        }
    }

    pub fn push_back(&self, t: *mut T) {
        assert_eq!(null_mut(), previous!(t).load(Relaxed));
        assert_eq!(null_mut(), next!(t).load(Relaxed));

        loop {
            if self
                .head
                .compare_exchange(null_mut(), t, Release, Relaxed)
                .is_ok()
            {
                self.tail
                    .compare_exchange(null_mut(), t, Release, Relaxed)
                    .ok();
                return;
            }

            let ltail = self.tail.load(Relaxed);
            previous!(t).store(ltail, Relaxed);
            if self
                .tail
                .compare_exchange(ltail, t, Release, Relaxed)
                .is_ok()
            {
                next!(ltail).store(t, Release);
                return;
            }
        }
    }

    pub fn pop_front(&self) -> Option<*mut T> {
        loop {
            let lhead = self.head.load(Relaxed);
            if lhead.is_null() {
                return None;
            }

            let lnext = next!(lhead).load(Relaxed);
            if self
                .head
                .compare_exchange(lhead, lnext, Release, Relaxed)
                .is_ok()
            {
                if lnext.is_null() {
                    self.tail
                        .compare_exchange(lhead, null_mut(), Release, Relaxed)
                        .ok();
                } else {
                    previous!(lnext).store(null_mut(), Release);
                }
                next!(lhead).store(null_mut(), Release);
                return Some(lhead);
            }
        }
    }

    pub fn append(&self, other: &Self) {
        'outer: loop {
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
            other
                .tail
                .compare_exchange(other_tail, null_mut(), Release, Relaxed)
                .ok();

            loop {
                let old_tail = self.tail.load(Relaxed);
                if old_tail.is_null() {
                    if self
                        .tail
                        .compare_exchange(old_tail, other_tail, Release, Relaxed)
                        .is_ok()
                    {
                        // If the tail was null, the head was null as well, so we need to update that.
                        // If the head changed, that means that someone else added something
                        // to the list, and we trust them to have updated the head pointer.
                        self.head
                            .compare_exchange(null_mut(), other_head, Release, Relaxed)
                            .ok();
                        return;
                    }
                } else if unsafe { (*old_tail).next() }
                    .compare_exchange(null_mut(), other_head, Release, Relaxed)
                    .is_ok()
                    && self
                        .tail
                        .compare_exchange(old_tail, other_tail, Release, Relaxed)
                        .is_ok()
                {
                    if unsafe { (*other_head).previous() }
                        .compare_exchange(null_mut(), old_tail, Release, Relaxed)
                        .is_err()
                    {
                        continue 'outer;
                    }
                    return;
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

    impl Node {
        fn into_mut_ptr(self) -> *mut Self {
            Box::into_raw(Box::new(self))
        }
    }

    #[kernel_test]
    fn test_append() {
        let node1 = Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 0,
        }
        .into_mut_ptr();
        let node2 = Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 1,
        }
        .into_mut_ptr();
        let node3 = Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 2,
        }
        .into_mut_ptr();
        let node4 = Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 3,
        }
        .into_mut_ptr();
        let node5 = Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 4,
        }
        .into_mut_ptr();
        let node6 = Node {
            next: AtomicPtr::default(),
            previous: AtomicPtr::default(),
            value: 5,
        }
        .into_mut_ptr();

        let list1 = LockFreeIntrusiveLinkedList::new();
        list1.push_back(node1);
        list1.push_back(node2);
        list1.push_back(node3);

        let list2 = LockFreeIntrusiveLinkedList::new();
        list2.push_back(node4);
        list2.push_back(node5);
        list2.push_back(node6);

        list1.append(&list2);

        assert_eq!(null_mut(), list2.head.load(Relaxed));
        assert_eq!(null_mut(), list2.tail.load(Relaxed));

        assert_eq!(node1, list1.head.load(Relaxed));
        assert_eq!(node6, list1.tail.load(Relaxed));

        assert_eq!(node2, unsafe { (*node1).next().load(Relaxed) });
        assert_eq!(node5, unsafe { (*node6).previous().load(Relaxed) });
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

        list.push_back(node1);
        assert_eq!(node1, list.head.load(Relaxed));
        assert_eq!(node1, list.tail.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).next().load(Relaxed) });

        list.push_back(node2);
        assert_eq!(node1, list.head.load(Relaxed));
        assert_eq!(node2, list.tail.load(Relaxed));
        assert_eq!(node2, unsafe { (*node1).next().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node2).next().load(Relaxed) });
        assert_eq!(node1, unsafe { (*node2).previous().load(Relaxed) });

        list.push_back(node3);
        assert_eq!(node1, list.head.load(Relaxed));
        assert_eq!(node3, list.tail.load(Relaxed));
        assert_eq!(node2, unsafe { (*node1).next().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(node3, unsafe { (*node2).next().load(Relaxed) });
        assert_eq!(node1, unsafe { (*node2).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node3).next().load(Relaxed) });
        assert_eq!(node2, unsafe { (*node3).previous().load(Relaxed) });

        let mut current = list.head.load(Relaxed);
        let mut count = 0;
        while !current.is_null() {
            assert_eq!(unsafe { (*current).value }, count);
            count += 1;
            unsafe {
                current = (*current).next.load(Relaxed);
            }
        }
        assert_eq!(count, 3);

        let node = list.pop_front();
        assert_eq!(node, Some(node1));
        assert_eq!(node2, list.head.load(Relaxed));
        assert_eq!(node3, list.tail.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).next().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node2).previous().load(Relaxed) });
        assert_eq!(node3, unsafe { (*node2).next().load(Relaxed) });
        assert_eq!(node2, unsafe { (*node3).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node3).next().load(Relaxed) });

        let node = list.pop_front();
        assert_eq!(node, Some(node2));
        assert_eq!(node3, list.head.load(Relaxed));
        assert_eq!(node3, list.tail.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).next().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node2).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node2).next().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node3).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node3).next().load(Relaxed) });

        let node = list.pop_front();
        assert_eq!(node, Some(node3));
        assert_eq!(null_mut(), list.head.load(Relaxed));
        assert_eq!(null_mut(), list.tail.load(Relaxed));
        assert_eq!(null_mut(), unsafe { (*node1).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node1).next().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node2).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node2).next().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node3).previous().load(Relaxed) });
        assert_eq!(null_mut(), unsafe { (*node3).next().load(Relaxed) });

        let node = list.pop_front();
        assert_eq!(node, None);
    }
}
