use std::collections::VecDeque;
use std::env::current_exe;
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

    fn keys(&self) -> Vec<&K> {
        self.links.iter().map(|(k, _)| k).collect::<Vec<_>>()
    }

    fn smallest_key(&self) -> &K {
        let link = self.links.first().unwrap();
        &link.0
    }

    fn insert_smallest_entry(&mut self, e: (K, NonNull<Node<K, V>>)) {
        self.links.insert(0, e);
    }

    fn remove_smallest_entry(&mut self) -> (K, NonNull<Node<K, V>>) {
        self.links.remove(0)
    }

    fn insert_largest_entry(&mut self, e: (K, NonNull<Node<K, V>>)) {
        self.links.push(e);
    }

    fn remove_largest_entry(&mut self) -> (K, NonNull<Node<K, V>>) {
        self.links.pop().unwrap()
    }

    fn lmerge_into(&mut self, other: &mut Internal<K, V>) {
        self.links.append(&mut other.links); // TODO: Should just use a VecDeque
        swap(&mut self.links, &mut other.links);
    }

    fn rmerge_into(&mut self, other: &mut Internal<K, V>) {
        other.links.append(&mut self.links);
    }

    fn is_root(&self) -> bool {
        self.parent.is_none()
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

        let key1 = child1.smallest_key();
        let key2 = child2.smallest_key();

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
    K: Ord + Debug,
{
    fn remove(&mut self, k: &K) -> Option<NonNull<Node<K, V>>> {
        let result = self.links.binary_search_by(|(key, _)| key.cmp(k));
        match result {
            Ok(index) => {
                let (k, v) = self.links.remove(index);
                Some(v)
            }
            Err(index) => None,
        }
    }

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

    fn left_index(&self, k: &K) -> Option<usize> {
        let mut index = self
            .links
            .binary_search_by(|(key, _)| key.cmp(k))
            .unwrap_or_else(|index| index);

        if index == 0 {
            return None;
        }

        index -= 1;

        Some(index)
    }

    fn left(&self, k: &K) -> Option<&K> {
        let (k, _) = self.left_entry(k)?;
        Some(k)
    }

    fn left_mut(&mut self, k: &K) -> Option<&mut K> {
        let (k, _) = self.left_entry_mut(k)?;
        Some(k)
    }

    fn left_entry(&self, k: &K) -> Option<&(K, NonNull<Node<K, V>>)> {
        let index = self.left_index(k)?;
        Some(&self.links[index])
    }

    fn left_entry_mut(&mut self, k: &K) -> Option<&mut (K, NonNull<Node<K, V>>)> {
        let index = self.left_index(k)?;
        Some(&mut self.links[index])
    }

    fn right_index(&self, k: &K) -> Option<usize> {
        let index = self.links.binary_search_by(|(key, _)| key.cmp(k));
        if let Ok(index) = index {
            if index >= (self.links.len() - 1) {
                return None;
            }
        }

        if let Err(index) = index {
            if index > (self.links.len() - 1) {
                return None;
            }
        }

        let index: usize = match index {
            Ok(index) => index + 1,
            Err(index) => index,
        };

        Some(index)
    }

    fn right(&self, k: &K) -> Option<&K> {
        let (k, _) = self.right_entry(k)?;
        Some(k)
    }

    fn right_mut(&mut self, k: &K) -> Option<&mut K> {
        let (k, _) = self.right_entry_mut(k)?;
        Some(k)
    }

    fn right_entry(&self, k: &K) -> Option<&(K, NonNull<Node<K, V>>)> {
        let index = self.right_index(k)?;
        Some(&self.links[index])
    }

    fn right_entry_mut(&mut self, k: &K) -> Option<&mut (K, NonNull<Node<K, V>>)> {
        let index = self.right_index(k)?;
        Some(&mut self.links[index])
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

    fn keys(&self) -> Vec<&K> {
        self.data.iter().map(|(k, _)| k).collect::<Vec<_>>()
    }

    fn smallest_key(&self) -> &K {
        let entry = self.data.first().unwrap();
        &entry.0
    }

    fn insert_smallest_entry(&mut self, e: (K, V)) {
        self.data.insert(0, e);
    }

    fn remove_smallest_entry(&mut self) -> (K, V) {
        self.data.remove(0)
    }

    fn insert_largest_entry(&mut self, e: (K, V)) {
        self.data.push(e);
    }

    fn remove_largest_entry(&mut self) -> (K, V) {
        self.data.pop().unwrap()
    }

    fn lmerge_into(&mut self, other: &mut Leaf<K, V>) {
        self.data.append(&mut other.data); // TODO: Should just use a VecDeque
        swap(&mut self.data, &mut other.data);
    }

    fn rmerge_into(&mut self, other: &mut Leaf<K, V>) {
        other.data.append(&mut self.data);
    }

    fn is_root(&self) -> bool {
        self.parent.is_none()
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
        let result = self.data.binary_search_by(|(key, _)| key.cmp(k));
        match result {
            Ok(index) => {
                let (k, v) = self.data.remove(index);
                Some(v)
            }
            Err(index) => None,
        }
    }

    fn find(&self, k: &K) -> Option<&(K, V)> {
        self.data.iter().find(|(key, _)| key == k)
    }

    fn find_mut(&mut self, k: &K) -> Option<&mut (K, V)> {
        self.data.iter_mut().find(|(key, _)| key == k)
    }
}

#[derive(Debug)]
enum NodeEntry<K, V> {
    Leaf((K, V)),
    Internal((K, NonNull<Node<K, V>>)),
}

#[derive(Debug)]
enum NodeValue<K, V> {
    Leaf(V),
    Internal(NonNull<Node<K, V>>),
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

    fn keys(&self) -> Vec<&K> {
        match self {
            Node::Internal(internal) => internal.keys(),
            Node::Leaf(leaf) => leaf.keys(),
        }
    }

    fn smallest_key(&self) -> &K {
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

    fn insert_smallest_entry(&mut self, e: NodeEntry<K, V>) {
        match (self, e) {
            (Node::Internal(internal), NodeEntry::Internal(e)) => {
                todo!()
            }
            (Node::Leaf(leaf), NodeEntry::Leaf(e)) => leaf.insert_smallest_entry(e),
            (Node::Leaf(..), NodeEntry::Internal(..)) => {
                panic!("Trying to insert Internal node entry into a Leaf!")
            }
            (Node::Internal(..), NodeEntry::Leaf(..)) => {
                panic!("Trying to insert Leaf entry into an Internal node!")
            }
        }
    }

    fn remove_smallest_entry(&mut self) -> NodeEntry<K, V> {
        match self {
            Node::Internal(internal) => {
                todo!()
            }
            Node::Leaf(leaf) => NodeEntry::Leaf(leaf.remove_smallest_entry()),
        }
    }

    fn insert_largest_entry(&mut self, e: NodeEntry<K, V>) {
        match (self, e) {
            (Node::Internal(internal), NodeEntry::Internal(e)) => {
                todo!()
            }
            (Node::Leaf(leaf), NodeEntry::Leaf(e)) => leaf.insert_largest_entry(e),
            (Node::Leaf(..), NodeEntry::Internal(..)) => {
                panic!("Trying to insert Internal node entry into a Leaf!")
            }
            (Node::Internal(..), NodeEntry::Leaf(..)) => {
                panic!("Trying to insert Leaf entry into an Internal node!")
            }
        }
    }

    fn remove_largest_entry(&mut self) -> NodeEntry<K, V> {
        match self {
            Node::Internal(internal) => {
                todo!()
            }
            Node::Leaf(leaf) => NodeEntry::Leaf(leaf.remove_largest_entry()),
        }
    }

    fn lmerge_into(&mut self, other: &mut Node<K, V>) {
        match self {
            Node::Internal(internal) => internal.lmerge_into(other.as_internal_mut()),
            Node::Leaf(leaf) => leaf.lmerge_into(other.as_leaf_mut()),
        }
    }

    fn rmerge_into(&mut self, other: &mut Node<K, V>) {
        match self {
            Node::Internal(internal) => internal.rmerge_into(other.as_internal_mut()),
            Node::Leaf(leaf) => leaf.rmerge_into(other.as_leaf_mut()),
        }
    }

    fn is_root(&self) -> bool {
        match self {
            Node::Internal(internal) => internal.is_root(),
            Node::Leaf(leaf) => leaf.is_root(),
        }
    }
}

impl<K, V> Node<K, V>
where
    K: Ord,
    K: Ord + Debug,
    V: Debug,
{
    fn update_key_from_smaller_to_bigger(&mut self, k: K) {
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

    fn update_key_from_bigger_to_smaller(&mut self, k: K) {
        match self {
            Node::Internal(internal) => {
                if let Some((key, ptr)) = internal.right_entry_mut(&k) {
                    *key = k;
                    return;
                }

                let (key, ptr) = internal.left_entry_mut(&k).unwrap();
                *key = k;
            }
            Node::Leaf(leaf) => {
                todo!()
            }
        }
    }

    fn left(&self, k: &K) -> Option<&K> {
        match self {
            Node::Internal(internal) => internal.left(k),
            Node::Leaf(leaf) => {
                todo!()
            }
        }
    }

    fn right(&self, k: &K) -> Option<&K> {
        match self {
            Node::Internal(internal) => internal.right(k),
            Node::Leaf(leaf) => {
                todo!()
            }
        }
    }

    fn remove_key(&mut self, k: &K) -> Option<NodeValue<K, V>> {
        match self {
            Node::Internal(internal) => {
                let v = internal.remove(k)?;
                Some(NodeValue::Internal(v))
            }
            Node::Leaf(leaf) => {
                let v = leaf.remove(k)?;
                Some(NodeValue::Leaf(v))
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
        self.internal_insert(k, v)
    }

    fn internal_insert(&mut self, k: K, v: V) -> Option<V> {
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
        let smallest_key = leaf.smallest_key();
        if &k < smallest_key {
            need_to_recursively_update_parents = true;
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

        if self.size == 0 {
            let root = self
                .root
                .expect("Size shrunk to 0, there MUST have been a root node before");
            let _ = unsafe { Box::from_raw(root.as_ptr()) };
            self.root = None;
            return value;
        }

        if leaf.is_root() {
            return value;
        }

        let mut need_to_recursively_update_parents = false;
        let smallest_key = leaf.smallest_key();
        if k < smallest_key {
            need_to_recursively_update_parents = true;
        }

        if need_to_recursively_update_parents {
            unsafe { self.update_parent_key_from_smaller_to_bigger(node_ptr) };
        }

        let size = unsafe { node_ptr.as_ref().size() }; // Miri Stacked Borrows rule violation without this line
        if size < self.min_node_size() {
            unsafe { self.transfer_or_merge(node_ptr) };
        }

        value
    }

    pub fn contains(&mut self, k: &K) -> bool {
        self.find(k).is_some()
    }

    pub fn find(&mut self, k: &K) -> Option<&V> {
        let leaf = self.find_leaf_node(k)?;
        let (_, v) = unsafe { leaf.as_ref() }.as_leaf().find(k)?;
        Some(v)
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

                let key = new.smallest_key();
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
                            let child = child_ptr.as_mut();
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

    unsafe fn transfer_or_merge(&mut self, mut node_ptr: NonNull<Node<K, V>>) {
        let node = unsafe { node_ptr.as_ref() };
        let k = node.smallest_key();
        let parent_ptr = node
            .parent()
            .expect("We should only be doing this on nodes with parent");
        let parent = unsafe { parent_ptr.as_ref() };

        // println!("parent: {parent:?}");
        let left_neighbour = parent.left(k);
        let right_neighbour = parent.right(k);

        println!("l neighbour: {left_neighbour:?} for k {k:?}");
        println!("r neighbour: {right_neighbour:?} for k {k:?}");

        print_bplustree(self, DebugOptions::default().all_address());
        print_ptr(node_ptr);

        if let Some(left) = left_neighbour {
            let neighbour_ptr = self.find_leaf_node(left).unwrap();
            let neighbour = unsafe { neighbour_ptr.as_ref() };
            if neighbour.size() > self.min_node_size() {
                unsafe { self.transfer(neighbour_ptr, node_ptr) };
                return;
            }
        }

        if let Some(right) = right_neighbour {
            let neighbour_ptr = self.find_leaf_node(right).unwrap();
            let neighbour = unsafe { neighbour_ptr.as_ref() };
            if neighbour.size() > self.min_node_size() {
                unsafe { self.transfer(node_ptr, neighbour_ptr) };
                return;
            }
        }

        // Merge
        if let Some(left) = left_neighbour {
            let neighbour_ptr = self.find_leaf_node(left).unwrap();
            unsafe { self.merge(neighbour_ptr, node_ptr) };
            return;
        }

        if let Some(right) = right_neighbour {
            let neighbour_ptr = self.find_leaf_node(right).unwrap();
            unsafe { self.merge(node_ptr, neighbour_ptr) };
            return;
        }

        let leaf = unsafe { node_ptr.as_mut().as_leaf_mut() };
        let (k, v) = leaf.remove_smallest_entry();
        unsafe {
            self.remove_key_from_hierarchy(node_ptr, &k);
        }
        assert!(
            self.internal_insert(k, v).is_none(),
            "There should be no entry for this key"
        );

        todo!()
    }

    unsafe fn transfer(
        &mut self,
        mut left_ptr: NonNull<Node<K, V>>,
        mut right_ptr: NonNull<Node<K, V>>,
    ) {
        println!("Transfer?");
        let left = unsafe { left_ptr.as_mut() };
        let right = unsafe { right_ptr.as_mut() };
        let l_size = left.size();
        let r_size = right.size();
        let current_ptr = if l_size < r_size {
            let entry = right.remove_smallest_entry();
            left.insert_largest_entry(entry);
            unsafe { self.update_parent_key_from_smaller_to_bigger(right_ptr) }
            right_ptr
        } else {
            let entry = left.remove_largest_entry();
            right.insert_smallest_entry(entry);
            unsafe { self.update_parent_key_from_bigger_to_smaller(right_ptr) }
            left_ptr
        };

        unsafe {
            self.handle_parent_size_change(current_ptr);
        }
    }

    unsafe fn merge(
        &mut self,
        mut left_ptr: NonNull<Node<K, V>>,
        mut right_ptr: NonNull<Node<K, V>>,
    ) {
        println!("Merge?");
        let left = unsafe { left_ptr.as_mut() };
        let right = unsafe { right_ptr.as_mut() };
        let l_size = left.size();
        let r_size = right.size();
        let current_ptr = if l_size < r_size {
            let k = left.smallest_key().clone();
            left.lmerge_into(right);
            unsafe {
                self.remove_key_from_single_parent(left_ptr, &k);
                self.update_parent_key_from_smaller_to_bigger(right_ptr);
            };
            right_ptr
        } else {
            let k = right.smallest_key().clone();
            right.rmerge_into(left);
            unsafe {
                self.remove_key_from_single_parent(right_ptr, &k);
                self.update_parent_key_from_smaller_to_bigger(left_ptr);
            };
            left_ptr
        };

        unsafe {
            self.handle_parent_size_change(current_ptr);
        }
    }

    unsafe fn handle_parent_size_change(&mut self, current_ptr: NonNull<Node<K, V>>) {
        let current = unsafe { current_ptr.as_ref() };
        if current.is_root() {
            todo!("Should just return here")
        }

        if let Some(parent_ptr) = current.parent() {
            let parent = unsafe { parent_ptr.as_ref() };
            if parent.size() < self.min_node_size() {
                self.transfer_or_merge(parent_ptr);
                print_bplustree(self, DebugOptions::default().all_address());
                print_ptr(parent_ptr);
                todo!()
            }
        }
    }

    unsafe fn update_parent_smallest_key(&self, mut node_ptr: NonNull<Node<K, V>>) {
        let node = unsafe { node_ptr.as_mut() };
        let smallest = node.smallest_key().clone();

        let mut current = node;
        while let Some(mut parent_ptr) = current.parent() {
            let parent = unsafe { parent_ptr.as_mut() };
            parent.set_smallest(smallest.clone());

            current = parent;
        }
    }

    unsafe fn update_parent_key_from_smaller_to_bigger(&self, mut node_ptr: NonNull<Node<K, V>>) {
        let node = unsafe { node_ptr.as_mut() };
        let mut current = node;
        while let Some(mut parent_ptr) = current.parent() {
            let smallest = current.smallest_key().clone();
            let parent = unsafe { parent_ptr.as_mut() };
            parent.update_key_from_smaller_to_bigger(smallest.clone());

            current = parent;
        }
    }

    unsafe fn update_parent_key_from_bigger_to_smaller(&self, mut node_ptr: NonNull<Node<K, V>>) {
        let node = unsafe { node_ptr.as_mut() };

        let mut current = node;
        while let Some(mut parent_ptr) = current.parent() {
            let parent = unsafe { parent_ptr.as_mut() };
            let smallest = current.smallest_key().clone();
            parent.update_key_from_bigger_to_smaller(smallest.clone());

            current = parent;
        }
    }

    unsafe fn remove_key_from_hierarchy(&self, mut node_ptr: NonNull<Node<K, V>>, k: &K) {
        let node = unsafe { node_ptr.as_mut() };
        let mut current = node;
        while let Some(mut parent_ptr) = current.parent() {
            let parent = unsafe { parent_ptr.as_mut() };
            unsafe {
                self.remove_key_from_node(parent_ptr, k);
            }
            current = parent;
        }
    }

    unsafe fn remove_key_from_single_parent(&self, mut node_ptr: NonNull<Node<K, V>>, k: &K) {
        let node = unsafe { node_ptr.as_mut() };
        if let Some(parent_ptr) = node.parent() {
            unsafe {
                self.remove_key_from_node(parent_ptr, k);
            }
        }
    }

    unsafe fn remove_key_from_node(&self, mut node_prt: NonNull<Node<K, V>>, k: &K) {
        let node = unsafe { node_prt.as_mut() };
        if let Some(NodeValue::Internal(ptr)) = node.remove_key(k) {
            let node = unsafe { ptr.as_ref() };
            let size = node.size();
            if size == 0 {
                let _ = unsafe { Box::from_raw(ptr.as_ptr()) };
            }
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

    let _ = tree
        .largest_key()
        .expect("If a tree is not empty, it's guaranteed to have at least a single value");

    unsafe { print_node(root, options) };
}

#[derive(Debug, Copy, Clone, Default)]
struct PtrDebugOptions {
    show_values: bool,
}

impl PtrDebugOptions {
    fn values(self) -> Self {
        Self { show_values: true }
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct ShowParent {
    internal: Option<PtrDebugOptions>,
    leaf: Option<PtrDebugOptions>,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DebugOptions {
    show_parent: ShowParent,
    override_padding: Option<usize>,
}

impl DebugOptions {
    fn internal_address(mut self) -> Self {
        let ptr_debug_options = if let Some(ptr_debug_options) = self.show_parent.internal {
            ptr_debug_options
        } else {
            PtrDebugOptions::default()
        };

        self.show_parent.internal = Some(ptr_debug_options);
        self
    }

    fn internal_values(mut self) -> Self {
        let ptr_debug_options = if let Some(ptr_debug_options) = self.show_parent.internal {
            ptr_debug_options.values()
        } else {
            PtrDebugOptions::default().values()
        };

        self.show_parent.internal = Some(ptr_debug_options);
        self
    }

    fn leaf_address(mut self) -> Self {
        let ptr_debug_options = if let Some(ptr_debug_options) = self.show_parent.leaf {
            ptr_debug_options
        } else {
            PtrDebugOptions::default().values()
        };

        self.show_parent.leaf = Some(ptr_debug_options);
        self
    }

    fn leaf_values(mut self) -> Self {
        let ptr_debug_options = if let Some(ptr_debug_options) = self.show_parent.leaf {
            ptr_debug_options.values()
        } else {
            PtrDebugOptions::default().values()
        };

        self.show_parent.leaf = Some(ptr_debug_options);
        self
    }

    fn all_address(self) -> Self {
        self.leaf_address().internal_address()
    }

    fn all_values(self) -> Self {
        self.internal_values().leaf_values()
    }

    fn override_padding(mut self, padding: usize) -> Self {
        self.override_padding = Some(padding);
        self
    }
}

unsafe fn print_node<K, V>(ptr: NonNull<Node<K, V>>, options: DebugOptions)
where
    K: Debug,
    V: Debug,
{
    let key_length = if let Some(padding) = options.override_padding {
        padding
    } else {
        4
    };
    let mut stack = VecDeque::from([(None, 0, false, ptr, -1)]);
    while let Some((k, mut offset, ignore_offset, current_ptr, lvl)) = stack.pop_front() {
        let current = unsafe { current_ptr.as_ref() };
        if let Some(key) = k {
            let line = if let Some(ptr_debug_options) = options.show_parent.internal {
                let formatted_ptr = if let Some(parent_ptr) = current.parent() {
                    unsafe { format_ptr(parent_ptr, ptr_debug_options) }
                } else {
                    "null".to_string()
                };
                format!("({}) {key:key_length$?}  ->  ", formatted_ptr)
            } else {
                format!("{key:key_length$?}  ->  ")
            };

            offset += line.chars().count();
            let mut offset = offset;
            if ignore_offset {
                offset = 0;
            }
            print!("{:>offset$}", line);
        }

        let mut should_print_new_line = false;

        match current {
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
                    let line = if let Some(ptr_debug_options) = options.show_parent.leaf {
                        let formatted_ptr = if let Some(parent) = leaf.parent {
                            unsafe { format_ptr(parent, ptr_debug_options) }
                        } else {
                            "null".to_string()
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

        if should_print_new_line && !stack.is_empty() {
            println!()
        }
    }
}

unsafe fn format_ptr<K, V>(ptr: NonNull<Node<K, V>>, ptr_debug_options: PtrDebugOptions) -> String
where
    K: Debug,
    V: Debug,
{
    let n = unsafe { &*ptr.as_ptr() };
    if !ptr_debug_options.show_values {
        return format!("({ptr:p})",);
    }

    let data = match n {
        Node::Internal(internal) => internal.links.iter().map(|(k, _)| k).collect::<Vec<_>>(),
        Node::Leaf(leaf) => leaf.data.iter().map(|(k, _)| k).collect::<Vec<_>>(),
    };

    format!("({ptr:p}): {:?}", data)
}

unsafe fn print_ptr<K, V>(ptr: NonNull<Node<K, V>>)
where
    K: Debug,
    V: Debug,
{
    unsafe {
        println!("{}", format_ptr(ptr, PtrDebugOptions::default().values()));
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
            use crate::bplustree::{BPlusTree, DebugOptions, print_bplustree};

            #[test]
            fn print_single_level() {
                let mut btree = BPlusTree::new(4);
                let options = DebugOptions::default().all_values();
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

                btree.insert((12345, 20), 4);

                println!();
                print_bplustree(&btree, options);
                println!();

                btree.insert((12345, 25), 5);

                println!();
                print_bplustree(&btree, options);
                println!();

                btree.insert((12345, 11), 6);

                println!();
                print_bplustree(&btree, options);
                println!();

                btree.insert((12345, 35), 7);

                println!();
                print_bplustree(&btree, options);
                println!();

                btree.insert((12345, 40), 8);

                println!();
                print_bplustree(&btree, options);
                println!();

                btree.insert((12345, 45), 9);

                println!();
                print_bplustree(&btree, options);
                println!();

                btree.insert((12345, 50), 10);

                println!();
                print_bplustree(&btree, options);
                println!();

                btree.insert((12345, 55), 11);

                println!();
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
                assert_eq!(root.smallest_key(), &17);

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
                assert_eq!(root.smallest_key(), &2);
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
                _               */

                let leaf1 = unsafe { btree.find_leaf_node(&(12345, 4)).unwrap().as_ref() };
                let leaf2 = unsafe { btree.find_leaf_node(&(12345, 14)).unwrap().as_ref() };
                assert_eq!(leaf1.parent(), leaf2.parent());

                btree.insert((12345, 22), 13);
                println!();
                print_bplustree(&btree, DebugOptions::default().leaf_address());
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
            use crate::bplustree::{BPlusTree, DebugOptions, print_bplustree};

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
            fn insert_and_remove_single_value() {
                let mut btree = BPlusTree::new(4);
                btree.insert(0, 0);
                assert_eq!(btree.remove(&0), Some(0));
                assert_eq!(btree.remove(&0), None);

                let mut iter = LevelIterator::new(&btree);
                let level1 = iter.next();
                assert!(level1.is_empty());
            }

            #[test]
            fn insert_two_remove_one() {
                let mut btree = BPlusTree::new(4);
                btree.insert(0, 0);
                btree.insert(5, 0);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                assert_eq!(btree.remove(&0), Some(0));

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);
                    let level1 = iter.next();

                    assert_eq!(level1.len(), 1);

                    let root = level1[0].as_leaf();

                    assert_eq!(root.data[0], (5, 0));
                    assert!(iter.next().is_empty());
                }
            }

            #[test]
            fn remove_and_transfer_1() {
                /*
                 (0:   0)
                 (5:   1)
                 (10:  2)
                 (15:  3)

                 Insert(20, 4)

                 (0)  ->  (0:   0)
                          (5:   1)

                 (10) ->  (10:  2)
                          (15:  3)
                          (20:  4)

                 Remove(0)

                                   LEFT

                 (5)  ->  (5:   1) // Smaller than min_node_size

                 (10) ->  (10:  2) RIGHT
                          (15:  3)
                          (20:  4)

                 Can't take one from left,
                 Can take one from Right

                 (5)  ->  (5:   1)
                          (10:  2)

                 (15) ->  (15:  3)
                          (20:  4)
                */

                let mut btree = BPlusTree::new(4);
                btree.insert(0, 0);
                btree.insert(5, 1);
                btree.insert(10, 2);
                btree.insert(15, 3);
                btree.insert(20, 4);

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();
                }

                btree.remove(&0);

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);
                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    assert_eq!(level1[0].keys(), vec![&5, &15]);

                    let level2 = iter.next();
                    assert_eq!(level2.len(), 2);
                    assert_eq!(level2[0].keys(), vec![&5, &10]);
                    assert_eq!(level2[1].keys(), vec![&15, &20]);
                }
            }

            #[test]
            fn remove_and_transfer_2() {
                let mut btree = BPlusTree::new(4);
                btree.insert(0, 0);
                btree.insert(5, 1);
                btree.insert(10, 2);
                btree.insert(15, 3);
                btree.insert(20, 4);
                btree.insert(7, 5);
                btree.remove(&10);

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);
                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    let root = level1[0].as_internal();
                    assert_eq!(root.links.len(), 2);
                    assert_eq!(root.links[0].0, 0);
                    assert_eq!(root.links[1].0, 15);
                }

                btree.remove(&15);

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);
                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    let root = level1[0].as_internal();
                    assert_eq!(root.links.len(), 2);
                    assert_eq!(root.links[0].0, 0);
                    assert_eq!(root.links[1].0, 7);
                }
            }

            #[test]
            fn remove_and_merge_1() {
                let mut btree = BPlusTree::new(4);
                btree.insert(0, 0);
                btree.insert(5, 1);
                btree.insert(15, 2);
                btree.insert(20, 3);
                btree.insert(7, 4);
                btree.insert(9, 5);
                btree.insert(30, 6);
                btree.insert(8, 7);
                btree.insert(6, 8);
                btree.remove(&7);
                btree.remove(&8);
                btree.remove(&6);

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);

                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    let root = level1[0];
                    let keys = root.keys();
                    assert_eq!(keys, vec![&0, &9, &20]);

                    let level2 = iter.next();
                    assert_eq!(level2.len(), 3);
                    let keys = level2[0].keys();
                    assert_eq!(keys, vec![&0, &5]);
                    let keys = level2[1].keys();
                    assert_eq!(keys, vec![&9, &15]);
                    let keys = level2[2].keys();
                    assert_eq!(keys, vec![&20, &30]);
                }

                btree.remove(&9);

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);

                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    let root = level1[0];
                    let keys = root.keys();
                    assert_eq!(keys, vec![&0, &20]);

                    let level2 = iter.next();
                    assert_eq!(level2.len(), 2);
                    let keys = level2[0].keys();
                    assert_eq!(keys, vec![&0, &5, &15]);
                    let keys = level2[1].keys();
                    assert_eq!(keys, vec![&20, &30]);
                }
            }

            #[test]
            fn remove_and_merge_2() {
                let mut btree = BPlusTree::new(4);
                btree.insert(0, 0);
                btree.insert(5, 1);
                btree.insert(15, 2);
                btree.insert(20, 3);
                btree.insert(7, 4);
                btree.insert(9, 5);
                btree.insert(30, 6);
                btree.insert(8, 7);
                btree.insert(6, 8);
                btree.remove(&7);
                btree.remove(&8);
                btree.remove(&6);

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);

                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    let root = level1[0];
                    let keys = root.keys();
                    assert_eq!(keys, vec![&0, &9, &20]);

                    let level2 = iter.next();
                    assert_eq!(level2.len(), 3);
                    let keys = level2[0].keys();
                    assert_eq!(keys, vec![&0, &5]);
                    let keys = level2[1].keys();
                    assert_eq!(keys, vec![&9, &15]);
                    let keys = level2[2].keys();
                    assert_eq!(keys, vec![&20, &30]);
                }

                btree.remove(&20);

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);

                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    let root = level1[0];
                    let keys = root.keys();
                    assert_eq!(keys, vec![&0, &9]);

                    let level2 = iter.next();
                    assert_eq!(level2.len(), 2);
                    let keys = level2[0].keys();
                    assert_eq!(keys, vec![&0, &5]);
                    let keys = level2[1].keys();
                    assert_eq!(keys, vec![&9, &15, &30]);
                }
            }

            #[test]
            fn remove_and_merge_3() {
                let mut btree = BPlusTree::new(4);
                for i in 0..=10 {
                    btree.insert(5 * i, i);
                }

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);

                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    let root = level1[0];
                    let keys = root.keys();
                    assert_eq!(keys, vec![&0, &20]);

                    let level2 = iter.next();
                    assert_eq!(level2.len(), 2);
                    let keys = level2[0].keys();
                    assert_eq!(keys, vec![&0, &10]);
                    let keys = level2[1].keys();
                    assert_eq!(keys, vec![&20, &30, &40]);

                    let level3 = iter.next();
                    assert_eq!(level3.len(), 5);
                    let keys = level3[0].keys();
                    assert_eq!(keys, vec![&0, &5]);
                    let keys = level3[1].keys();
                    assert_eq!(keys, vec![&10, &15]);
                    let keys = level3[2].keys();
                    assert_eq!(keys, vec![&20, &25]);
                    let keys = level3[3].keys();
                    assert_eq!(keys, vec![&30, &35]);
                    let keys = level3[4].keys();
                    assert_eq!(keys, vec![&40, &45, &50]);
                }

                btree.remove(&25);

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);

                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    let root = level1[0];
                    let keys = root.keys();
                    assert_eq!(keys, vec![&0, &20]);

                    let level2 = iter.next();
                    assert_eq!(level2.len(), 2);
                    let keys = level2[0].keys();
                    assert_eq!(keys, vec![&0, &10]);
                    let keys = level2[1].keys();
                    assert_eq!(keys, vec![&20, &40]);

                    let level3 = iter.next();
                    assert_eq!(level3.len(), 4);
                    let keys = level3[0].keys();
                    assert_eq!(keys, vec![&0, &5]);
                    let keys = level3[1].keys();
                    assert_eq!(keys, vec![&10, &15]);
                    let keys = level3[2].keys();
                    assert_eq!(keys, vec![&20, &30, &35]);
                    let keys = level3[3].keys();
                    assert_eq!(keys, vec![&40, &45, &50]);
                }
            }

            #[test]
            fn remove_and_collapse_1() {
                let mut btree = BPlusTree::new(4);
                for i in 0..=10 {
                    btree.insert(5 * i, i);
                }

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&0);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&5);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&10);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&15);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                assert!(false, "this needs a proper assert");
            }

            #[test]
            fn remove_and_collapse_2() {
                let mut btree = BPlusTree::new(4);
                for i in 0..=10 {
                    btree.insert(5 * i, i);
                }

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&20);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&25);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&30);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&35);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&40);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&45);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&50);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                assert!(false, "this needs a proper assert");
            }

            #[test]
            fn remove_and_collapse_3() {
                let mut btree = BPlusTree::new(4);
                for i in 0..=10 {
                    btree.insert(5 * i, i);
                }

                btree.insert(16, 11);
                btree.insert(17, 12);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&20);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&25);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&30);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&35);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&40);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&45);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                // btree.remove(&50);
                //
                // println!();
                // print_bplustree(&btree, DebugOptions::default());
                // println!();

                assert!(false, "this needs a proper assert");
            }

            #[test]
            fn remove_and_collapse_4() {
                let mut btree = BPlusTree::new(4);
                for i in 0..=10 {
                    btree.insert(5 * i, i);
                }

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&0);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();
            }

            #[test]
            fn remove_from_leaf_node_that_is_3_levels_down() {
                let mut btree = BPlusTree::new(4);
                for i in 0..=10 {
                    btree.insert(5 * i, i);
                }

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);

                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    let root = level1[0];
                    let keys = root.keys();
                    assert_eq!(keys, vec![&0, &20]);

                    let level2 = iter.next();
                    assert_eq!(level2.len(), 2);
                    let keys = level2[0].keys();
                    assert_eq!(keys, vec![&0, &10]);
                    let keys = level2[1].keys();
                    assert_eq!(keys, vec![&20, &30, &40]);

                    let level3 = iter.next();
                    assert_eq!(level3.len(), 5);
                    let keys = level3[0].keys();
                    assert_eq!(keys, vec![&0, &5]);
                    let keys = level3[1].keys();
                    assert_eq!(keys, vec![&10, &15]);
                    let keys = level3[2].keys();
                    assert_eq!(keys, vec![&20, &25]);
                    let keys = level3[3].keys();
                    assert_eq!(keys, vec![&30, &35]);
                    let keys = level3[4].keys();
                    assert_eq!(keys, vec![&40, &45, &50]);
                }

                btree.remove(&40);

                {
                    println!();
                    print_bplustree(&btree, DebugOptions::default());
                    println!();

                    let mut iter = LevelIterator::new(&btree);

                    let level1 = iter.next();
                    assert_eq!(level1.len(), 1);
                    let root = level1[0];
                    let keys = root.keys();
                    assert_eq!(keys, vec![&0, &20]);

                    let level2 = iter.next();
                    assert_eq!(level2.len(), 2);
                    let keys = level2[0].keys();
                    assert_eq!(keys, vec![&0, &10]);
                    let keys = level2[1].keys();
                    assert_eq!(keys, vec![&20, &30, &45]);

                    let level3 = iter.next();
                    assert_eq!(level3.len(), 5);
                    let keys = level3[0].keys();
                    assert_eq!(keys, vec![&0, &5]);
                    let keys = level3[1].keys();
                    assert_eq!(keys, vec![&10, &15]);
                    let keys = level3[2].keys();
                    assert_eq!(keys, vec![&20, &25]);
                    let keys = level3[3].keys();
                    assert_eq!(keys, vec![&30, &35]);
                    let keys = level3[4].keys();
                    assert_eq!(keys, vec![&45, &50]);
                }
            }

            #[test]
            fn remove_smallest_should_update_the_parent() {
                let mut btree = BPlusTree::new(4);
                btree.insert(0, 0);
                btree.insert(5, 1);
                btree.insert(10, 2);
                btree.insert(15, 3);
                btree.insert(20, 4);

                {
                    println!();
                    print_bplustree(&btree, Default::default());
                    println!();

                    let mut level_iter = LevelIterator::new(&btree);
                    let level1 = level_iter.next();
                    let root_links = &level1[0].as_internal().links;
                    assert_eq!(root_links[0].0, 0);
                    assert_eq!(root_links[1].0, 10);
                }

                assert_eq!(btree.remove(&10), Some(2));

                {
                    println!();
                    print_bplustree(&btree, Default::default());
                    println!();

                    let mut level_iter = LevelIterator::new(&btree);
                    let level1 = level_iter.next();
                    let root_links = &level1[0].as_internal().links;
                    assert_eq!(root_links[0].0, 0);
                    assert_eq!(root_links[1].0, 15);
                }
            }
        }

        mod fuzz {
            use crate::bplustree::{BPlusTree, DebugOptions, print_bplustree};
            use rand::{random_bool, random_range};

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
                print_bplustree(&btree, DebugOptions::default());
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
                print_bplustree(&btree, DebugOptions::default());
                println!();
                println!("size: {}", btree.size());
            }

            #[test]
            #[ignore = "Non-deterministic"]
            fn insert_and_remove_at_random() {
                let mut btree = BPlusTree::new(4);
                let n = 500;
                for i in 0..n {
                    let k = random_range(0..=25);

                    let insert = random_bool(0.4);
                    if insert {
                        btree.insert(k, i);
                    } else {
                        btree.remove(&k);
                    }
                }

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();
                println!("size: {}", btree.size());
            }

            #[test]
            fn t() {
                let mut btree = BPlusTree::new(4);
                btree.insert(13, 0);
                btree.remove(&0);
                btree.insert(3, 2);
                btree.remove(&13);
                btree.remove(&17);
                btree.insert(11, 5);
                btree.remove(&15);
                btree.insert(19, 7);
                btree.insert(25, 8);
                btree.remove(&14);
                btree.remove(&11);
                btree.insert(16, 11);
                btree.remove(&16);
                btree.insert(20, 13);
                btree.remove(&13);
                btree.remove(&17);
                btree.remove(&24);
                btree.remove(&14);
                btree.insert(14, 18);
                btree.insert(12, 19);
                btree.insert(14, 20);
                btree.remove(&22);
                btree.insert(22, 22);
                btree.remove(&5);
                btree.insert(24, 24);
                btree.remove(&19);
                btree.remove(&3);
                btree.remove(&22);
                btree.insert(15, 28);
                btree.remove(&5);
                btree.remove(&12);
                btree.remove(&1);
                btree.remove(&17);
                btree.remove(&3);
                btree.insert(12, 34);
                btree.remove(&7);
                btree.insert(18, 36);
                btree.remove(&11);
                btree.insert(20, 38);
                btree.insert(23, 39);
                btree.remove(&18);
                btree.insert(17, 41);
                btree.remove(&20);
                btree.remove(&15);
                btree.insert(14, 44);
                btree.remove(&11);
                btree.remove(&18);
                btree.insert(1, 47);
                btree.remove(&25);
                btree.insert(25, 49);
                btree.insert(24, 50);
                btree.remove(&5);
                btree.remove(&19);
                btree.remove(&19);
                btree.remove(&17);
                btree.insert(15, 55);
                btree.remove(&7);
                btree.remove(&1);
                btree.insert(12, 58);
                btree.remove(&11);
                btree.remove(&19);
                btree.remove(&1);
                btree.remove(&5);
                btree.insert(10, 63);
                btree.insert(20, 64);
                btree.remove(&6);
                btree.remove(&18);
                btree.insert(17, 67);
                btree.remove(&8);
                btree.remove(&1);
                btree.remove(&13);
                btree.remove(&4);
                btree.insert(4, 72);
                btree.remove(&12);
                btree.remove(&7);
                btree.remove(&11);
                btree.remove(&7);
                btree.insert(21, 77);
                btree.remove(&8);
                btree.insert(10, 79);
                btree.remove(&10);
                btree.insert(10, 81);
                btree.remove(&17);
                btree.remove(&24);
                btree.remove(&1);
                btree.remove(&10);
                btree.insert(7, 86);
                btree.insert(15, 87);
                btree.remove(&22);
                btree.remove(&13);
                btree.insert(4, 90);
                btree.remove(&19);
                btree.remove(&16);
                btree.insert(22, 93);
                btree.insert(8, 94);
                btree.insert(0, 95);
                btree.insert(16, 96);
                btree.remove(&19);
                btree.remove(&25);
                btree.remove(&3);
                btree.remove(&18);
                btree.remove(&20);
                btree.remove(&1);
                btree.remove(&23);
                btree.remove(&25);
                btree.remove(&6);
                btree.remove(&19);
                btree.remove(&13);
                btree.insert(21, 108);
                btree.remove(&13);
                btree.insert(6, 110);
                btree.remove(&25);
                btree.insert(14, 112);
                btree.insert(13, 113);
                btree.insert(19, 114);
                btree.insert(6, 115);
                btree.remove(&3);
                btree.remove(&5);
                btree.remove(&13);
                btree.remove(&19);
                btree.remove(&16);
                btree.remove(&18);
                btree.remove(&18);
                btree.insert(9, 123);
                btree.remove(&4);
                btree.insert(23, 125);
                btree.remove(&7);
                btree.remove(&14);
                btree.remove(&22);
                btree.remove(&14);
                btree.insert(24, 130);
                btree.remove(&4);
                btree.remove(&3);
                btree.insert(24, 133);
                btree.insert(3, 134);
                btree.remove(&18);
                btree.remove(&19);
                btree.insert(8, 137);
                btree.insert(17, 138);
                btree.remove(&3);
                btree.remove(&13);
                btree.remove(&18);
                btree.insert(0, 142);
                btree.insert(2, 143);
                btree.remove(&4);
                btree.insert(19, 145);
                btree.insert(22, 146);
                btree.insert(20, 147);
                btree.insert(5, 148);
                btree.remove(&15);
                btree.remove(&16);
                btree.remove(&16);
                btree.insert(18, 152);
                btree.remove(&21);
                btree.insert(23, 154);
                btree.insert(0, 155);
                btree.insert(15, 156);
                btree.remove(&23);
                btree.insert(8, 158);
                btree.remove(&15);
                btree.remove(&8);
                btree.remove(&23);
                btree.remove(&13);
                btree.remove(&19);
                btree.remove(&13);
                btree.insert(4, 165);
                btree.insert(9, 166);
                btree.insert(13, 167);
                btree.remove(&9);
                btree.remove(&13);
                btree.remove(&3);
                btree.insert(15, 171);
                btree.remove(&2);
                btree.insert(6, 173);
                btree.insert(25, 174);
                btree.remove(&22);
                btree.remove(&1);
                btree.remove(&12);
                btree.remove(&2);
                btree.remove(&22);
                btree.remove(&14);
                btree.insert(25, 181);
                btree.remove(&24);
                btree.insert(5, 183);
                btree.insert(21, 184);
                btree.remove(&25);
                btree.insert(15, 186);
                btree.remove(&15);
                btree.insert(5, 188);
                btree.remove(&6);
                btree.insert(18, 190);
                btree.remove(&17);
                btree.insert(7, 192);
                btree.remove(&3);
                btree.insert(15, 194);
                btree.remove(&2);
                btree.remove(&12);
                btree.remove(&9);
                btree.insert(15, 198);
                btree.insert(25, 199);
                btree.remove(&22);
                btree.insert(11, 201);
                btree.insert(1, 202);
                btree.insert(6, 203);
                btree.remove(&4);
                btree.remove(&6);
                btree.remove(&8);
                btree.remove(&6);
                btree.insert(6, 208);
                btree.insert(7, 209);
                btree.insert(24, 210);
                btree.remove(&20);
                btree.remove(&5);
                btree.remove(&4);
                btree.remove(&4);
                btree.remove(&18);
                btree.remove(&1);
                btree.insert(9, 217);
                btree.remove(&14);
                btree.insert(16, 219);
                btree.remove(&18);
                btree.insert(18, 221);
                btree.remove(&16);
                btree.remove(&14);
                btree.remove(&22);
                btree.remove(&8);
                btree.remove(&17);
                btree.remove(&7);
                btree.insert(0, 228);
                btree.remove(&8);
                btree.remove(&24);
                btree.remove(&7);
                btree.insert(22, 232);
                btree.remove(&10);
                btree.remove(&12);
                btree.insert(19, 235);
                btree.remove(&17);
                btree.remove(&16);
                btree.remove(&1);
                btree.remove(&20);
                btree.insert(3, 240);
                btree.remove(&14);
                btree.insert(2, 242);
                btree.remove(&12);
                btree.insert(15, 244);
                btree.remove(&3);
                btree.remove(&11);
                btree.remove(&19);
                btree.insert(1, 248);
                btree.remove(&13);
                btree.remove(&20);
                btree.remove(&13);
                btree.remove(&2);
                btree.remove(&16);
                btree.remove(&13);
                btree.insert(4, 255);
                btree.remove(&12);
                btree.insert(21, 257);
                btree.insert(19, 258);
                btree.insert(24, 259);
                btree.insert(11, 260);
                btree.remove(&19);
                btree.insert(2, 262);
                btree.remove(&9);
                btree.remove(&7);
                btree.remove(&3);
                btree.remove(&17);
                btree.insert(17, 267);
                btree.remove(&20);
                btree.remove(&8);
                btree.remove(&2);
                btree.remove(&14);
                btree.insert(20, 272);
                btree.remove(&13);
                btree.insert(9, 274);
                btree.insert(11, 275);
                btree.insert(24, 276);
                btree.remove(&9);
                btree.insert(5, 278);
                btree.remove(&12);
                btree.remove(&10);
                btree.remove(&9);
                btree.remove(&10);
                btree.remove(&0);
                btree.insert(20, 284);
                btree.remove(&12);
                btree.insert(7, 286);
                btree.insert(24, 287);
                btree.remove(&22);
                btree.insert(13, 289);
                btree.remove(&19);
                btree.remove(&13);
                btree.insert(4, 292);
                btree.remove(&19);
                btree.insert(2, 294);
                btree.remove(&22);
                btree.remove(&5);
                btree.remove(&21);
                btree.remove(&14);
                btree.remove(&0);
                btree.remove(&19);
                btree.remove(&7);
                btree.insert(4, 302);
                btree.insert(9, 303);
                btree.remove(&16);
                btree.remove(&5);
                btree.remove(&20);
                btree.insert(22, 307);
                btree.remove(&0);
                btree.remove(&7);
                btree.remove(&4);
                btree.insert(19, 311);
                btree.insert(20, 312);
                btree.remove(&5);
                btree.remove(&20);
                btree.remove(&13);
                btree.insert(16, 316);
                btree.remove(&5);
                btree.remove(&21);
                btree.insert(9, 319);
                btree.remove(&23);
                btree.remove(&13);
                btree.insert(7, 322);
                btree.remove(&6);
                btree.insert(21, 324);
                btree.remove(&22);
                btree.remove(&10);
                btree.remove(&18);
                btree.remove(&13);
                btree.insert(23, 329);
                btree.remove(&17);
                btree.remove(&11);
                btree.remove(&8);
                btree.insert(13, 333);
                btree.remove(&24);
                btree.remove(&15);
                btree.remove(&7);
                btree.insert(13, 337);
                btree.insert(19, 338);
                btree.remove(&18);
                btree.remove(&3);
                btree.insert(4, 341);
                btree.remove(&24);
                btree.remove(&19);
                btree.remove(&2);
                btree.remove(&1);
                btree.insert(6, 346);
                btree.insert(2, 347);
                btree.insert(14, 348);
                btree.remove(&8);
                btree.insert(15, 350);
                btree.insert(2, 351);
                btree.remove(&12);
                btree.remove(&20);
                btree.remove(&4);
                btree.remove(&19);
                btree.insert(25, 356);
                btree.remove(&0);
                btree.remove(&6);
                btree.remove(&17);
                btree.remove(&7);
                btree.remove(&16);
                btree.insert(21, 362);
                btree.insert(3, 363);
                btree.remove(&10);
                btree.remove(&17);
                btree.remove(&9);
                btree.insert(11, 367);
                btree.insert(15, 368);
                btree.insert(16, 369);
                btree.remove(&19);
                btree.insert(24, 371);
                btree.insert(5, 372);
                btree.remove(&7);
                btree.remove(&5);
                btree.insert(9, 375);
                btree.insert(18, 376);
                btree.remove(&24);
                btree.remove(&14);
                btree.insert(9, 379);
                btree.remove(&3);
                btree.remove(&12);
                btree.remove(&22);
                btree.remove(&6);
                btree.remove(&8);
                btree.remove(&1);
                btree.remove(&11);
                btree.insert(25, 387);
                btree.remove(&24);
                btree.remove(&3);
                btree.remove(&15);
                btree.insert(9, 391);
                btree.remove(&23);
                btree.remove(&15);
                btree.insert(13, 394);
                btree.remove(&5);
                btree.remove(&23);
                btree.insert(6, 397);
                btree.remove(&6);
                btree.remove(&4);
                btree.insert(22, 400);
                btree.remove(&11);
                btree.remove(&8);
                btree.remove(&8);
                btree.remove(&18);
                btree.remove(&13);
                btree.insert(14, 406);
                btree.remove(&8);
                btree.insert(24, 408);
                btree.insert(6, 409);
                btree.remove(&16);
                btree.insert(11, 411);
                btree.insert(14, 412);
                btree.insert(8, 413);
                btree.remove(&11);
                btree.remove(&19);
                btree.remove(&13);
                btree.remove(&24);
                btree.remove(&11);
                btree.remove(&9);
                btree.insert(24, 420);
                btree.remove(&25);
                btree.remove(&1);
                btree.remove(&7);
                btree.insert(12, 424);
                btree.insert(22, 425);
                btree.remove(&15);
                btree.insert(3, 427);
                btree.insert(3, 428);
                btree.remove(&7);
                btree.remove(&24);
                btree.remove(&14);
                btree.insert(19, 432);
                btree.insert(12, 433);
                btree.remove(&10);
                btree.remove(&16);
                btree.remove(&4);
                btree.insert(7, 437);
                btree.remove(&18);

                println!();
                print_bplustree(&btree, DebugOptions::default());
                println!();

                btree.remove(&21);
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

        #[test]
        fn left_1() {
            let leaf1 = create_leaf(0, 0);
            let leaf2 = create_leaf(5, 1);
            let leaf3 = create_leaf(10, 2);
            let leaf4 = create_leaf(15, 3);
            let leaf5 = create_leaf(20, 4);
            let leaf6 = create_leaf(25, 5);

            let internal = Internal {
                parent: None,
                links: vec![
                    (0, leaf1),
                    (5, leaf2),
                    (10, leaf3),
                    (15, leaf4),
                    (20, leaf5),
                    (25, leaf6),
                ],
            };

            assert_eq!(internal.left(&-1), None);
            assert_eq!(internal.left(&0), None);
            assert_eq!(internal.left(&2), Some(&0));
            assert_eq!(internal.left(&3), Some(&0));
            assert_eq!(internal.left(&4), Some(&0));
            assert_eq!(internal.left(&5), Some(&0));
            assert_eq!(internal.left(&10), Some(&5));
            assert_eq!(internal.left(&25), Some(&20));
            assert_eq!(internal.left(&30), Some(&25));

            unsafe {
                cleanup_leaf(leaf1);
                cleanup_leaf(leaf2);
                cleanup_leaf(leaf3);
                cleanup_leaf(leaf4);
                cleanup_leaf(leaf5);
                cleanup_leaf(leaf6);
            }
        }

        #[test]
        fn right_1() {
            let leaf1 = create_leaf(0, 0);
            let leaf2 = create_leaf(5, 1);
            let leaf3 = create_leaf(10, 2);
            let leaf4 = create_leaf(15, 3);
            let leaf5 = create_leaf(20, 4);
            let leaf6 = create_leaf(25, 5);

            let internal = Internal {
                parent: None,
                links: vec![
                    (0, leaf1),
                    (5, leaf2),
                    (10, leaf3),
                    (15, leaf4),
                    (20, leaf5),
                    (25, leaf6),
                ],
            };

            assert_eq!(internal.right(&-1), Some(&0));
            assert_eq!(internal.right(&0), Some(&5));
            assert_eq!(internal.right(&2), Some(&5));
            assert_eq!(internal.right(&3), Some(&5));
            assert_eq!(internal.right(&4), Some(&5));
            assert_eq!(internal.right(&5), Some(&10));
            assert_eq!(internal.right(&10), Some(&15));
            assert_eq!(internal.right(&25), None);
            assert_eq!(internal.right(&30), None);

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
