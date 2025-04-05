use std::collections::VecDeque;
use std::fmt::{Debug, Pointer};
use std::mem;
use std::mem::swap;
use std::ptr::NonNull;

#[derive(Debug)]
struct Internal<K, V> {
    parent: Option<NonNull<Node<K, V>>>,
    links: Vec<(K, NonNull<Node<K, V>>)>,
}

impl<K, V> Internal<K, V> {
    fn split(&mut self) -> NonNull<Node<K, V>> {
        let right = self.links.split_off(self.links.len() / 2);
        assert!(self.links.len() <= right.len());

        unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node::Internal(Internal {
                parent: None,
                links: right,
            }))))
        }
    }

    fn size(&self) -> usize {
        self.links.len()
    }
}

impl<K, V> Internal<K, V>
where
    K: Clone + PartialOrd + Debug,
{
    unsafe fn new_with_children(
        mut child1_ptr: NonNull<Node<K, V>>,
        mut child2_ptr: NonNull<Node<K, V>>,
    ) -> NonNull<Node<K, V>> {
        let child1 = unsafe { child1_ptr.as_mut() };
        let child2 = unsafe { child2_ptr.as_mut() };

        let key1 = child1.smallest_key().unwrap();
        let key2 = child2.smallest_key().unwrap();

        // assert!(key1 <= key2, "key1: {key1:?} | key2: {key2:?}");

        let internal_node = Node::Internal(Internal {
            parent: None,
            links: vec![(key1.clone(), child1_ptr), (key2.clone(), child2_ptr)],
        });

        let internal_ptr =
            unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(internal_node))) };

        unsafe {
            child1.set_parent(internal_ptr);
            child2.set_parent(internal_ptr);
        }

        internal_ptr
    }
}

impl<K, V> Internal<K, V>
where
    K: Ord,
{
    fn find(&self, k: &K) -> NonNull<Node<K, V>> {
        assert!(
            !self.links.is_empty(),
            "An internal Node must have children"
        );

        let index = self
            .links
            .binary_search_by(|(key, _)| key.cmp(k))
            .unwrap_or_else(|index| if index == 0 { index } else { index - 1 });

        self.links[index].1
    }
}

#[derive(Debug)]
struct Leaf<K, V> {
    parent: Option<NonNull<Node<K, V>>>,
    data: Vec<(K, V)>,
}

impl<K, V> Leaf<K, V> {
    fn new() -> Self {
        Self {
            parent: None,
            data: vec![],
        }
    }

    fn split(&mut self) -> NonNull<Node<K, V>> {
        let right = self.data.split_off(self.data.len() / 2);
        assert!(self.data.len() <= right.len());

        unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node::Leaf(Leaf {
                parent: None,
                data: right,
            }))))
        }
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

impl<K, V> Leaf<K, V>
where
    K: Ord,
{
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        let result = self.data.binary_search_by(|(key, _)| key.cmp(&k));
        let mut pair = (k, v);
        match result {
            Ok(index) => {
                swap(&mut self.data[index], &mut pair);
                Some(pair.1)
            }
            Err(index) => {
                self.data.insert(index, pair);
                None
            }
        }
    }
}

#[derive(Debug)]
enum Node<K, V> {
    Internal(Internal<K, V>),
    Leaf(Leaf<K, V>),
}

impl<K, V> Node<K, V> {
    fn parent(&self) -> Option<NonNull<Node<K, V>>> {
        match self {
            Node::Internal(internal) => internal.parent,
            Node::Leaf(leaf) => leaf.parent,
        }
    }

    /// SAFETY:
    ///  * ptr MUST NOT point to self
    ///  * ptr MUST NOT be dangling
    unsafe fn set_parent(&mut self, ptr: NonNull<Node<K, V>>) {
        match self {
            Node::Internal(internal) => internal.parent = Some(ptr),
            Node::Leaf(leaf) => leaf.parent = Some(ptr),
        }
    }

    fn smallest_key(&self) -> Option<&K> {
        match self {
            Node::Internal(internal) => {
                let link = internal.links.get(0)?;
                Some(&link.0)
            }
            Node::Leaf(leaf) => {
                let entry = leaf.data.get(0)?;
                Some(&entry.0)
            }
        }
    }

    fn largest_key(&self) -> Option<&K> {
        match self {
            Node::Internal(internal) => internal.links.last().map(|(k, _)| k),
            Node::Leaf(leaf) => leaf.data.last().map(|(k, _)| k),
        }
    }

    fn set_first(&mut self, k: K) {
        match self {
            Node::Internal(internal) => {
                if let Some(first) = internal.links.get_mut(0) {
                    first.0 = k;
                }
            }
            Node::Leaf(leaf) => {
                if let Some(first) = leaf.data.get_mut(0) {
                    first.0 = k;
                }
            }
        }
    }

    fn size(&self) -> usize {
        match self {
            Node::Internal(internal) => internal.size(),
            Node::Leaf(leaf) => leaf.size(),
        }
    }
}

#[derive(Debug)]
struct BPlusTree<K, V> {
    order: usize,
    root: Option<NonNull<Node<K, V>>>,
    size: usize,
}

impl<K, V> BPlusTree<K, V> {
    fn size(&self) -> usize {
        self.size
    }

    fn largest_key(&self) -> Option<&K> {
        let current = self.root?;
        let mut queue = VecDeque::from([current]);
        while let Some(current) = queue.pop_front() {
            match unsafe { current.as_ref() } {
                Node::Internal(internal) => {
                    let (_, ptr) = internal
                        .links
                        .last()
                        .expect("An Internal node MUST have a child");
                    queue.push_front(*ptr);
                }
                Node::Leaf(_) => {
                    break;
                }
            }
        }

        unsafe { current.as_ref().largest_key() }
    }
}

impl<K, V> BPlusTree<K, V>
where
    K: Clone + Ord + Debug,
    V: Ord + Debug,
{
    fn new(order: usize) -> Self {
        Self {
            order,
            root: None,
            size: 0,
        }
    }

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        println!("btree.insert({k:?}, {v:?});");
        if self.root.is_none() {
            let mut leaf = Leaf::new();
            leaf.data.push((k, v));
            let ptr = Box::into_raw(Box::new(Node::Leaf(leaf)));
            self.root = Some(unsafe { NonNull::new_unchecked(ptr) });

            self.size = 1;
            return None;
        }

        let mut leaf_ptr = self.find_leaf_node(&k).unwrap(); // SAFETY: We checked that root is not None
        let node = unsafe { leaf_ptr.as_mut() };
        let Node::Leaf(leaf) = node else {
            unreachable!();
        };

        let entry = leaf.insert(k.clone(), v);

        let new_entry = entry.is_none();
        if new_entry {
            self.size += 1;
        }

        if let Some(mut parent_ptr) = leaf.parent {
            // Is parent's first node key value higher than what we've just inserted? Then update it.
            let parent = unsafe { parent_ptr.as_mut() };
            let first = parent.smallest_key().unwrap(); // SAFETY: This Node is our parent, therefore it MUST have at least one value.
            let should_update_parents_first = *first > k;
            if should_update_parents_first {
                // Propagate first key update to all parents up (tested by: smallest_key_update_should_propagate_to_all_parents)
                let mut queue = VecDeque::from([parent_ptr]);
                while let Some(mut current_ptr) = queue.pop_front() {
                    let current = unsafe { current_ptr.as_mut() };
                    current.set_first(k.clone());

                    if let Some(parent_ptr) = current.parent() {
                        queue.push_front(parent_ptr);
                    }
                }
            }
        }

        if leaf.size() > self.max_node_size() {
            let new_leaf_ptr = leaf.split();
            unsafe {
                self.insert_into_parent_node(leaf_ptr, new_leaf_ptr);
            }
        }

        entry
    }

    fn find_leaf_node(&self, k: &K) -> Option<NonNull<Node<K, V>>> {
        let root = self.root?;

        let mut current = root;
        loop {
            let node = unsafe { current.as_ref() };
            let Node::Internal(internal) = node else {
                break;
            };

            current = internal.find(k);
        }

        Some(current)
    }

    fn remove(&mut self, k: &K) -> Option<V> {
        todo!()
    }

    fn contains(&mut self, k: &K) -> bool {
        todo!()
    }

    fn find(&mut self, k: &K) -> Option<V> {
        todo!()
    }

    fn max_node_size(&self) -> usize {
        self.order // This BPlusTree is slightly different, each ENTRY in internal node points to a child, not the LINKS between entries
    }

    unsafe fn insert_into_parent_node(
        &mut self,
        old_ptr: NonNull<Node<K, V>>,
        mut new_ptr: NonNull<Node<K, V>>,
    ) {
        let old = unsafe { old_ptr.as_ref() };
        if let Some(mut parent_ptr) = old.parent() {
            unsafe {
                let new = new_ptr.as_mut();

                new.set_parent(parent_ptr);

                let parent = parent_ptr.as_mut();
                let Node::Internal(parent) = parent else {
                    unreachable!("Leaf node cannot be a parent of another node.")
                };

                let key = new.smallest_key().unwrap();
                let index = parent
                    .links
                    .binary_search_by(|(k, _)| k.cmp(key))
                    .expect_err(&format!(
                        "The parent node MUST NOT have this value: {key:?}"
                    ));
                parent.links.insert(index, (key.clone(), new_ptr));

                let need_to_split_parent = parent.size() > self.max_node_size();
                if need_to_split_parent {
                    let mut split_off_from_parent_ptr = parent.split();

                    // If parent node needed to be split as well, update it's children to point to it
                    unsafe {
                        let Node::Internal(split_off_from_parent) =
                            split_off_from_parent_ptr.as_mut()
                        else {
                            unreachable!(
                                "This can't be a leaf node, as it's a parent of another node"
                            )
                        };

                        // TODO: Could this be simplified and is there a performance cost to iterating over every single link? (they ARE at most max_node_size())
                        for (_, child_ptr) in &mut split_off_from_parent.links {
                            let child = unsafe { child_ptr.as_mut() };
                            child.set_parent(split_off_from_parent_ptr);
                        }
                    }

                    self.insert_into_parent_node(parent_ptr, split_off_from_parent_ptr)
                }
            }
        } else {
            let parent_ptr = unsafe { Internal::new_with_children(old_ptr, new_ptr) };
            self.root = Some(parent_ptr);
        }
    }
}

impl<K, V> Drop for BPlusTree<K, V> {
    fn drop(&mut self) {
        let Some(current) = self.root else { return };

        let mut queue = VecDeque::from([current]);
        unsafe {
            while let Some(mut current) = queue.pop_front() {
                if let Node::Internal(internal) = current.as_mut() {
                    let mut links = vec![];
                    swap(&mut internal.links, &mut links);
                    for (_, ptr) in links {
                        queue.push_back(ptr);
                    }
                }

                let _ = Box::from_raw(current.as_ptr());
            }
        }
    }
}

fn print_bplustree<K, V>(tree: &BPlusTree<K, V>, options: DebugOptions)
where
    K: Debug,
    V: Debug,
{
    let Some(root) = tree.root else {
        println!("Empty");
        return;
    };

    let key = tree
        .largest_key()
        .expect("If a tree is not empty, it's guaranteed to have at least a single value");

    unsafe { print_node(root, options) };
}

#[derive(Debug, Copy, Clone, Default)]
struct DebugOptions {
    show_parent: bool,
}

unsafe fn print_node<K, V>(ptr: NonNull<Node<K, V>>, options: DebugOptions)
where
    K: Debug,
    V: Debug,
{
    let key_length = 4;
    let mut stack = VecDeque::from([(None, 0, false, ptr, -1)]);
    while let Some((k, mut offset, ignore_offset, current, lvl)) = stack.pop_front() {
        if let Some(key) = k {
            let line = format!("{key:key_length$?}  ->  ");
            offset += line.chars().count();
            let mut offset = offset;
            if ignore_offset {
                offset = 0;
            }
            print!("{:>offset$}", line);
        }

        let mut should_print_new_line = false;

        let node = unsafe { current.as_ref() };
        match node {
            Node::Internal(internal) => {
                for (index, (k, ptr)) in internal.links.iter().rev().enumerate() {
                    let last = index == internal.links.len() - 1;
                    let mut ignore_offset = false;
                    if last {
                        ignore_offset = true
                    }
                    stack.push_front((Some(k), offset, ignore_offset, *ptr, lvl + 1));
                }
            }
            Node::Leaf(leaf) => {
                let mut first = true;
                for (k, v) in &leaf.data {
                    let line = if options.show_parent {
                        let formatted_ptr = if let Some(parent) = leaf.parent {
                            unsafe { format_ptr(leaf.parent.unwrap()) }
                        } else {
                            "No parent".to_string()
                        };
                        format!("({}) {k:key_length$?}: {v:key_length$?}", formatted_ptr)
                    } else {
                        format!("{k:key_length$?}: {v:key_length$?}")
                    };

                    let mut offset = offset + line.chars().count();
                    if first {
                        offset = 0;
                        first = false;
                    }
                    println!("{line:>offset$}");
                }

                should_print_new_line = true;
            }
        }

        if should_print_new_line {
            println!()
        }
    }
}

unsafe fn format_ptr<K, V>(ptr: NonNull<Node<K, V>>) -> String
where
    K: Debug,
    V: Debug,
{
    let n = &*ptr.as_ptr();
    match n {
        Node::Internal(internal) => {
            format!(
                "({ptr:p}): {:?}",
                internal
                    .links
                    .iter()
                    .map(|(k, v)| { k })
                    .collect::<Vec<_>>()
            )
        }
        Node::Leaf(leaf) => {
            format!(
                "({ptr:p}): {:?}",
                leaf.data.iter().map(|(k, v)| { k }).collect::<Vec<_>>()
            )
        }
    }
}

unsafe fn print_ptr<K, V>(ptr: NonNull<Node<K, V>>)
where
    K: Debug,
    V: Debug,
{
    println!("{}", format_ptr(ptr));
}

#[cfg(test)]
mod tests {
    use crate::bplustree::{Leaf, Node};
    use std::ptr::NonNull;

    mod bplustree {
        use crate::bplustree::{BPlusTree, DebugOptions, Node, print_bplustree};

        mod print {
            use crate::bplustree::{BPlusTree, print_bplustree};

            #[test]
            fn print_single_level() {
                let mut btree = BPlusTree::new(4);
                let options = Default::default();
                btree.insert((12345, 0), 0);
                btree.insert((12345, 5), 1);
                btree.insert((12345, 10), 2);
                btree.insert((12345, 15), 3);
                print_bplustree(&btree, options);
            }

            #[test]
            fn print_three_levels() {
                let mut btree = BPlusTree::new(4);
                let options = Default::default();
                for i in 0..10 {
                    btree.insert((12345, 5 * i), i);
                }

                print_bplustree(&btree, options);

                println!();
                btree.insert((12345, 50), 10);
                println!();

                print_bplustree(&btree, options);
            }
        }

        #[test]
        fn insert_single_value() {
            let mut btree = BPlusTree::new(4);
            assert_eq!(btree.insert((12345, 1), 0), None);
        }

        #[test]
        fn insert_multiple_values() {
            let mut btree = BPlusTree::new(4);
            let options = Default::default();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 0), 0);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 5), 1);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 10), 2);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 15), 3);
            println!();
            print_bplustree(&btree, options);
            println!();
        }

        #[test]
        fn insert_a_lot_of_values() {
            let mut btree = BPlusTree::new(4);
            let options = Default::default();
            btree.insert((12345, 0), 0);
            println!();
            print_bplustree(&btree, options);

            btree.insert((12345, 5), 1);
            println!();
            print_bplustree(&btree, options);

            btree.insert((12345, 10), 2);
            println!();
            print_bplustree(&btree, options);

            btree.insert((12345, 15), 3);
            println!();
            print_bplustree(&btree, options);

            btree.insert((12345, 20), 4);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 25), 5);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 11), 6);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 35), 7);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 40), 8);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 45), 9);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 50), 10);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 55), 11);
            print_bplustree(&btree, options);
            println!();
        }

        #[test]
        fn updating_parent_to_smaller_value_on_regular_insert_1() {
            /*

            (12345,   32)  ->  (12345,   32):    2
                               (12345,   33):    1
                               (12345,   57):    5

            (12345,   78)  ->  (12345,   78):    0
                               (12345,   91):    4
                               (12345,   93):    3
                               (12345,   97):    6

            Insert((12345,   13):    7)

            Where should this insert go?

            I think into (12345, 32) and update the parent to (12345, 13)

            */

            let mut btree = BPlusTree::new(4);
            let options = Default::default();
            btree.insert((12345, 78), 0);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 33), 1);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 32), 2);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 93), 3);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 91), 4);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 57), 5);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 97), 6);
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 13), 7);
            print_bplustree(&btree, options);
            println!();
        }

        #[test]
        fn updating_parent_to_smaller_value_on_regular_insert_2() {
            let mut btree = BPlusTree::new(4);
            let options = Default::default();
            btree.insert((12345, 78), 0);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 33), 1);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 32), 2);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 93), 3);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 91), 4);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 57), 5);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 97), 6);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 13), 7);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 10), 8);
            println!();
            print_bplustree(&btree, options);
            println!();
        }

        #[test]
        fn smallest_key_update_should_propagate_to_all_parents() {
            let mut btree = BPlusTree::new(4);
            let args = [
                (191, 0),
                (173, 1),
                (143, 2),
                (158, 3),
                (45, 4),
                (133, 5),
                (76, 6),
                (95, 7),
                (31, 8),
                (134, 9),
                (118, 10),
                (17, 11),
                (20, 12),
                (74, 13),
            ];

            for (k, v) in args {
                btree.insert(k, v);
            }

            /*
             17  ->    17  ->    17:   11
                                 20:   12

                       31  ->    31:    8
                                 45:    4
                                 74:   13
                                 76:    6

             95  ->    95  ->    95:    7
                                118:   10

                      133  ->   133:    5
                                134:    9
                                143:    2

                      158  ->   158:    3
                                173:    1
                                191:    0
            */

            let root = unsafe { btree.root.unwrap().as_ref() };
            assert_eq!(root.smallest_key(), Some(&17));

            btree.insert(2, 14);

            /*
               2  ->    2  ->      2:   14
                                  17:   11
                                  20:   12

                        31  ->    31:    8
                                  45:    4
                                  74:   13
                                  76:    6

              95  ->    95  ->    95:    7
                                 118:   10

                       133  ->   133:    5
                                 134:    9
                                 143:    2

                       158  ->   158:    3
                                 173:    1
                                 191:    0
            */

            let root = unsafe { btree.root.unwrap().as_ref() };
            assert_eq!(root.smallest_key(), Some(&2));
        }

        #[test]
        fn parent_of_adjacent_nodes_is_updated_correctly_after_split() {
            let mut btree = BPlusTree::new(4);
            let options = DebugOptions { show_parent: false };
            btree.insert((12345, 4), 0);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 14), 1);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 6), 2);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 36), 3);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 7), 4);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 51), 5);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 12), 6);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 48), 7);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 50), 8);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 14), 9);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 42), 10);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 4), 11);
            println!();
            print_bplustree(&btree, options);
            println!();

            btree.insert((12345, 18), 12);
            println!();
            print_bplustree(&btree, options);
            println!();

            /*
             (12345,    4)  ->  (12345,    4):   11
                                (12345,    6):    2

             (12345,    7)  ->  (12345,    7):    4
                                (12345,   12):    6

             (12345,   14)  ->  (12345,   14):    9
                                (12345,   18):   12
                                (12345,   36):    3
                                (12345,   42):   10

             (12345,   48)  ->  (12345,   48):    7
                                (12345,   50):    8
                                (12345,   51):    5
            */

            let leaf1 = unsafe { btree.find_leaf_node(&(12345, 4)).unwrap().as_ref() };
            let leaf2 = unsafe { btree.find_leaf_node(&(12345, 14)).unwrap().as_ref() };
            assert_eq!(leaf1.parent(), leaf2.parent());

            btree.insert((12345, 22), 13);
            println!();
            print_bplustree(&btree, DebugOptions { show_parent: true });
            println!();

            /*
             (12345,    4)  ->  (12345,    4)  ->  (12345,    4):   11
                                                   (12345,    6):    2

                                (12345,    7)  ->  (12345,    7):    4
                                                   (12345,   12):    6

             (12345,   14)  ->  (12345,   14)  ->  (12345,   14):    9
                                                   (12345,   18):   12

                                (12345,   22)  ->  (12345,   22):   13
                                                   (12345,   36):    3
                                                   (12345,   42):   10

                                (12345,   48)  ->  (12345,   48):    7
                                                   (12345,   50):    8
            */

            let leaf1 = unsafe { btree.find_leaf_node(&(12345, 4)).unwrap().as_ref() };
            let leaf2 = unsafe { btree.find_leaf_node(&(12345, 14)).unwrap().as_ref() };
            assert_ne!(leaf1.parent(), leaf2.parent());
        }

        mod fuzz {
            use crate::bplustree::{BPlusTree, DebugOptions, print_bplustree};
            use rand::random_range;

            #[test]
            #[ignore = "Long and non-deterministic"]
            fn insert_random_strings() {
                let mut btree = BPlusTree::new(4);
                let n = 100;
                for i in 0..n {
                    let r0 = random_range(b'A'..=b'Z') as char;
                    let r1 = random_range(b'A'..=b'Z') as char;
                    let r2 = random_range(b'A'..=b'Z') as char;
                    let r3 = random_range(b'A'..=b'Z') as char;
                    let r4 = random_range(b'A'..=b'Z') as char;
                    let r5 = random_range(b'A'..=b'Z') as char;
                    let r6 = random_range(b'A'..=b'Z') as char;

                    btree.insert(format!("{r0}{r1}{r2}{r3}{r4}{r5}{r6}"), i);
                }

                println!();
                print_bplustree(&btree, DebugOptions { show_parent: false });
                println!();
                println!("size: {}", btree.size());
            }

            #[test]
            #[ignore = "Long and non-deterministic"]
            fn insert_random_pairs() {
                let mut btree = BPlusTree::new(4);
                let n = 100;
                for i in 0..n {
                    let i0 = random_range(0..=50);
                    let i1 = random_range(0..=50);

                    btree.insert((i0, i1), i);
                }

                println!();
                print_bplustree(&btree, DebugOptions { show_parent: false });
                println!();
                println!("size: {}", btree.size());
            }
        }

        #[test]
        fn find_leaf_node_1() {
            let mut btree = BPlusTree::new(4);
            btree.insert((12345, 1), 0);
            let node = unsafe { btree.find_leaf_node(&(12345, 2)).unwrap().as_ref() };
            let Node::Leaf(leaf) = node else {
                unreachable!()
            };
            assert_eq!(leaf.data[0], ((12345, 1), 0));
        }

        #[test]
        fn find_leaf_node_2() {
            let mut btree = BPlusTree::new(4);
            btree.insert((12345, 1), 0);
            btree.insert((12345, 3), 1);
            btree.insert((12345, 5), 2);
            let node = unsafe { btree.find_leaf_node(&(12345, 2)).unwrap().as_ref() };
            let Node::Leaf(leaf) = node else {
                unreachable!()
            };
            assert_eq!(leaf.data[0], ((12345, 1), 0));
        }
    }

    fn create_leaf<K, V>(k: K, v: V) -> NonNull<Node<K, V>> {
        let leaf = Node::Leaf(Leaf {
            parent: None,
            data: vec![(k, v)],
        });
        unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(leaf))) }
    }

    unsafe fn cleanup_leaf<K, V>(ptr: NonNull<Node<K, V>>) {
        let _ = Box::from_raw(ptr.as_ptr());
    }

    mod internal {
        use crate::bplustree::Internal;
        use crate::bplustree::tests::{cleanup_leaf, create_leaf};

        #[test]
        fn find() {
            let leaf1 = create_leaf((12345, 0), 0);
            let leaf2 = create_leaf((12345, 5), 1);
            let leaf3 = create_leaf((12345, 10), 2);
            let leaf4 = create_leaf((12345, 15), 3);
            let leaf5 = create_leaf((12345, 20), 4);
            let leaf6 = create_leaf((12345, 25), 5);

            let internal = Internal {
                parent: None,
                links: vec![
                    ((12345, 0), leaf1),
                    ((12345, 5), leaf2),
                    ((12345, 10), leaf3),
                    ((12345, 15), leaf4),
                    ((12345, 20), leaf5),
                    ((12345, 25), leaf6),
                ],
            };

            let node = internal.find(&(12345, 8));
            assert_eq!(node, leaf2);

            unsafe {
                cleanup_leaf(leaf1);
                cleanup_leaf(leaf2);
                cleanup_leaf(leaf3);
                cleanup_leaf(leaf4);
                cleanup_leaf(leaf5);
                cleanup_leaf(leaf6);
            }
        }
    }

    mod leaf {
        use crate::bplustree::tests::cleanup_leaf;
        use crate::bplustree::{Leaf, print_ptr};

        #[test]
        fn split_1() {
            let mut leaf = Leaf::new();
            leaf.insert((12345, 0), 0);
            leaf.insert((12345, 5), 1);
            leaf.insert((12345, 10), 2);
            leaf.insert((12345, 15), 3);
            leaf.insert((12345, 20), 4);
            leaf.insert((12345, 25), 5);
            leaf.insert((12345, 30), 6);
            leaf.insert((12345, 35), 7);
            let new_leaf = leaf.split();
            assert_eq!(leaf.size(), 4);
            unsafe {
                print_ptr(new_leaf);
                cleanup_leaf(new_leaf);
            }
            println!("{leaf:?}");
        }
    }

    #[test]
    fn search() {
        let v = vec![
            ((1234, 0), 0),
            ((1234, 5), 1),
            ((1234, 10), 2),
            ((1234, 15), 3),
            ((1234, 20), 4),
            ((1234, 25), 5),
            ((1234, 30), 6),
            ((1234, 35), 7),
        ];

        let key = (1234, 0);

        let index = v
            .binary_search_by(|(k, v)| {
                let ord = k.cmp(&key);
                println!("{:?}.cmp({:?}) -> {:?}", &k, &key, ord);
                ord
            })
            .unwrap_or_else(|index| index - 1);

        println!("{index:?}");
    }
}
