use std::collections::VecDeque;
use std::fmt::Debug;
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

    fn smallest_key(&self) -> Option<&K> {
        let link = self.links.first()?;
        Some(&link.0)
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

        debug_assert!(key1 <= key2, "key1: {key1:?} | key2: {key2:?}");

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
    fn find(&self, k: &K) -> &(K, NonNull<Node<K, V>>) {
        debug_assert!(
            !self.links.is_empty(),
            "An internal Node must have children"
        );

        let index = self
            .links
            .binary_search_by(|(key, _)| key.cmp(k))
            .unwrap_or_else(|index| if index == 0 { index } else { index - 1 });

        &self.links[index]
    }

    fn find_mut(&mut self, k: &K) -> &mut (K, NonNull<Node<K, V>>) {
        debug_assert!(
            !self.links.is_empty(),
            "An internal Node must have children"
        );

        let index = self
            .links
            .binary_search_by(|(key, _)| key.cmp(k))
            .unwrap_or_else(|index| if index == 0 { index } else { index - 1 });

        &mut self.links[index]
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
        debug_assert!(self.data.len() <= right.len());

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

    fn smallest_key(&self) -> Option<&K> {
        let entry = self.data.first()?;
        Some(&entry.0)
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

    fn remove(&mut self, k: &K) -> Option<V> {
        let result = self.data.binary_search_by(|(key, _)| key.cmp(&k));
        match result {
            Ok(index) => {
                let (k, v) = self.data.remove(index);
                Some(v)
            }
            Err(index) => None,
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
            Node::Internal(internal) => internal.smallest_key(),
            Node::Leaf(leaf) => leaf.smallest_key(),
        }
    }

    fn largest_key(&self) -> Option<&K> {
        match self {
            Node::Internal(internal) => internal.links.last().map(|(k, _)| k),
            Node::Leaf(leaf) => leaf.data.last().map(|(k, _)| k),
        }
    }

    fn set_smallest(&mut self, k: K) {
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

    fn as_internal(&self) -> &Internal<K, V> {
        match self {
            Node::Internal(internal) => internal,
            Node::Leaf(_leaf) => {
                panic!("Expected an Internal node but got Leaf")
            }
        }
    }

    fn as_internal_mut(&mut self) -> &mut Internal<K, V> {
        match self {
            Node::Internal(internal) => internal,
            Node::Leaf(_leaf) => {
                panic!("Expected an Internal node but got Leaf")
            }
        }
    }

    fn as_leaf(&self) -> &Leaf<K, V> {
        match self {
            Node::Internal(_internal) => {
                panic!("Expected a Leaf node but got Internal")
            }
            Node::Leaf(leaf) => leaf,
        }
    }

    fn as_leaf_mut(&mut self) -> &mut Leaf<K, V> {
        match self {
            Node::Internal(_internal) => {
                panic!("Expected a Leaf node but got Internal")
            }
            Node::Leaf(leaf) => leaf,
        }
    }
}

impl<K, V> Node<K, V>
where
    K: Ord,
{
    fn update_key(&mut self, k: K) {
        match self {
            Node::Internal(internal) => {
                let entry = internal.find_mut(&k);
                (*entry).0 = k;
            }
            Node::Leaf(leaf) => {
                todo!()
            }
        }
    }
}

#[derive(Debug)]
pub struct BPlusTree<K, V> {
    order: usize,
    root: Option<NonNull<Node<K, V>>>,
    size: usize,
}

impl<K, V> BPlusTree<K, V> {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn largest_key(&self) -> Option<&K> {
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
    pub fn new(order: usize) -> Self {
        Self {
            order,
            root: None,
            size: 0,
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        println!("btree.insert({k:?}, {v:?});");
        if self.root.is_none() {
            let mut leaf = Leaf::new();
            leaf.insert(k, v);
            let ptr = Box::into_raw(Box::new(Node::Leaf(leaf)));
            self.root = Some(unsafe { NonNull::new_unchecked(ptr) });

            self.size = 1;
            return None;
        }

        let mut node_ptr = self.find_leaf_node(&k).unwrap(); // SAFETY: We checked that root is not None
        let leaf = unsafe { node_ptr.as_mut().as_leaf_mut() };

        let mut need_to_recursively_update_parents = false;
        if let Some(smallest_key) = leaf.smallest_key() {
            if &k < smallest_key {
                need_to_recursively_update_parents = true;
            }
        }

        let value = leaf.insert(k, v);
        if value.is_none() {
            self.size += 1;
        }

        if need_to_recursively_update_parents {
            unsafe { self.update_parent_smallest_key(node_ptr) };
        }

        let leaf = unsafe { node_ptr.as_mut().as_leaf_mut() }; // Miri Stacked Borrows rule violation without this line
        if leaf.size() > self.max_node_size() {
            let new_leaf_ptr = leaf.split();
            unsafe {
                self.insert_into_parent_node(node_ptr, new_leaf_ptr);
            }
        }

        value
    }

    fn find_leaf_node(&self, k: &K) -> Option<NonNull<Node<K, V>>> {
        let root = self.root?;

        let mut current = root;
        loop {
            let node = unsafe { current.as_ref() };
            let Node::Internal(internal) = node else {
                break;
            };

            current = internal.find(k).1;
        }

        Some(current)
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        println!("btree.remove(&{k:?});");
        let mut node_ptr = self.find_leaf_node(k)?;
        let leaf = unsafe { node_ptr.as_mut().as_leaf_mut() };

        let value = leaf.remove(k);
        if value.is_some() {
            self.size -= 1;
        }

        let mut need_to_recursively_update_parents = false;
        if let Some(smallest_key) = leaf.smallest_key() {
            if k < smallest_key {
                need_to_recursively_update_parents = true;
            }
        }

        if need_to_recursively_update_parents {
            unsafe { self.update_parent_key(node_ptr) };
        }

        let leaf = unsafe { node_ptr.as_mut().as_leaf_mut() }; // Miri Stacked Borrows rule violation without this line
        if leaf.size() < self.min_node_size() {
            print_bplustree(self, DebugOptions::default());
            todo!()
        }

        value
    }

    pub fn contains(&mut self, k: &K) -> bool {
        self.find(k).is_some()
    }

    pub fn find(&mut self, k: &K) -> Option<&V> {
        todo!()
    }

    fn max_node_size(&self) -> usize {
        self.order // This BPlusTree is slightly different, each ENTRY in internal node points to a child, not the LINKS between entries
    }

    fn min_node_size(&self) -> usize {
        self.order / 2
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

    unsafe fn update_parent_smallest_key(&self, mut node_ptr: NonNull<Node<K, V>>) {
        let node = unsafe { node_ptr.as_mut() };
        let Some(smallest) = node.smallest_key().cloned() else {
            return;
        };

        let mut current = node;
        while let Some(mut parent_ptr) = current.parent() {
            let parent = unsafe { parent_ptr.as_mut() };
            parent.set_smallest(smallest.clone());

            current = parent;
        }
    }

    unsafe fn update_parent_key(&self, mut node_ptr: NonNull<Node<K, V>>) {
        let node = unsafe { node_ptr.as_mut() };
        let Some(smallest) = node.smallest_key().cloned() else {
            return;
        };

        let mut current = node;
        while let Some(mut parent_ptr) = current.parent() {
            let parent = unsafe { parent_ptr.as_mut() };
            parent.update_key(smallest.clone());

            current = parent;
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

pub fn print_bplustree<K, V>(tree: &BPlusTree<K, V>, options: DebugOptions)
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
pub struct DebugOptions {
    pub show_parent: bool,
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
    unsafe {
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
}

unsafe fn print_ptr<K, V>(ptr: NonNull<Node<K, V>>)
where
    K: Debug,
    V: Debug,
{
    unsafe {
        println!("{}", format_ptr(ptr));
    }
}

#[cfg(test)]
mod tests {
    use crate::bplustree::{BPlusTree, Leaf, Node};
    use std::collections::VecDeque;
    use std::ptr::NonNull;

    struct LevelIterator<'a, K, V> {
        btree: &'a BPlusTree<K, V>,
        queue: VecDeque<NonNull<Node<K, V>>>,
    }

    impl<'a, K, V> LevelIterator<'a, K, V> {
        fn new(btree: &'a BPlusTree<K, V>) -> LevelIterator<'a, K, V> {
            let mut queue = VecDeque::new();
            if let Some(root) = btree.root {
                queue.push_front(root)
            }

            LevelIterator { btree, queue }
        }

        fn next(&mut self) -> Vec<&Node<K, V>> {
            let mut next: VecDeque<NonNull<Node<K, V>>> = VecDeque::new();

            let mut output = vec![];
            while let Some(current_ptr) = self.queue.pop_front() {
                let current = unsafe { current_ptr.as_ref() };
                output.push(current);

                if let Node::Internal(internal) = current {
                    for (k, ptr) in &internal.links {
                        next.push_back(*ptr)
                    }
                }
            }

            self.queue = next;

            output
        }
    }

    mod bplustree {
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

        mod insert {
            use crate::bplustree::{BPlusTree, DebugOptions, print_bplustree};

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
                let options = DebugOptions::default();
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
        }

        mod remove {
            use crate::bplustree::tests::LevelIterator;
            use crate::bplustree::{BPlusTree, Internal, Node, print_bplustree};

            #[test]
            fn remove_on_empty() {
                let mut btree: BPlusTree<i32, i32> = BPlusTree::new(4);

                assert_eq!(btree.remove(&0), None);
            }

            #[test]
            fn remove_1() {
                let mut btree = BPlusTree::new(4);
                btree.insert(0, 0);
                btree.insert(5, 1);
                btree.insert(10, 2);
                btree.insert(15, 3);

                assert_eq!(btree.remove(&0), Some(0));
                assert_eq!(btree.remove(&0), None);
                assert_eq!(btree.remove(&20), None);
                assert_eq!(btree.size(), 3);
            }

            #[test]
            fn remove_and_force_join() {
                let mut btree = BPlusTree::new(4);
                btree.insert(0, 0);
                btree.insert(5, 1);
                btree.insert(10, 2);
                btree.insert(15, 3);
                btree.insert(20, 4);

                print_bplustree(&btree, Default::default());

                btree.remove(&10);
                btree.remove(&0);
                btree.remove(&20);

                print_bplustree(&btree, Default::default());
            }

            #[test]
            fn remove_smallest_should_update_the_parent() {
                let mut btree = BPlusTree::new(4);
                btree.insert(0, 0);
                btree.insert(5, 1);
                btree.insert(10, 2);
                btree.insert(15, 3);
                btree.insert(20, 4);

                let mut level_iter = LevelIterator::new(&btree);
                let level1 = level_iter.next();
                let root_links = &level1[0].as_internal().links;
                assert_eq!(root_links[0].0, 0);
                assert_eq!(root_links[1].0, 10);

                print_bplustree(&btree, Default::default());
                assert_eq!(btree.remove(&10), Some(2));
                print_bplustree(&btree, Default::default());

                let mut level_iter = LevelIterator::new(&btree);
                let level1 = level_iter.next();
                let root_links = &level1[0].as_internal().links;
                assert_eq!(root_links[0].0, 0);
                assert_eq!(root_links[1].0, 15);
            }
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
                let mut btree = BPlusTree::new(10);
                let n = 1_000_000;
                for i in 0..n {
                    let i0 = random_range(0..=1000);
                    let i1 = random_range(0..=1000);
                    let i2 = random_range(0..=60);

                    btree.insert((i0, i1, i2), i);
                }

                println!();
                print_bplustree(&btree, DebugOptions { show_parent: false });
                println!();
                println!("size: {}", btree.size());
            }
        }

        mod find {
            use crate::bplustree::{BPlusTree, Node};

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
    }

    fn create_leaf<K, V>(k: K, v: V) -> NonNull<Node<K, V>> {
        let leaf = Node::Leaf(Leaf {
            parent: None,
            data: vec![(k, v)],
        });
        unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(leaf))) }
    }

    unsafe fn cleanup_leaf<K, V>(ptr: NonNull<Node<K, V>>) {
        unsafe {
            let _ = Box::from_raw(ptr.as_ptr());
        }
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

            let (_, node) = *internal.find(&(12345, 8));
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
        let v = [
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
