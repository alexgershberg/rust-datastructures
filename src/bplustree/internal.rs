use crate::bplustree::node::Node;
use std::fmt::Debug;
use std::mem::swap;
use std::ptr::NonNull;

#[derive(Debug)]
pub struct Internal<K, V> {
    pub(crate) parent: Option<NonNull<Node<K, V>>>,
    pub(crate) links: Vec<(K, NonNull<Node<K, V>>)>,
}

impl<K, V> Internal<K, V>
where
    K: Ord + PartialOrd + Clone + Debug,
    V: Ord + PartialOrd + Clone + Debug,
{
    pub fn split(&mut self) -> NonNull<Node<K, V>> {
        let right = self.links.split_off(self.links.len() / 2);
        assert!(self.links.len() <= right.len());

        unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node::Internal(Internal {
                parent: None,
                links: right,
            }))))
        }
    }

    pub fn size(&self) -> usize {
        self.links.len()
    }

    pub fn keys(&self) -> Vec<&K> {
        self.links.iter().map(|(k, _)| k).collect::<Vec<_>>()
    }

    fn smallest_entry(&self) -> &(K, NonNull<Node<K, V>>) {
        self.links.first().unwrap()
    }

    fn smallest_entry_mut(&mut self) -> &mut (K, NonNull<Node<K, V>>) {
        self.links.first_mut().unwrap()
    }

    pub fn smallest_key(&self) -> &K {
        &self.smallest_entry().0
    }

    pub fn smallest_value(&self) -> NonNull<Node<K, V>> {
        self.smallest_entry().1
    }

    pub fn set_smallest(&mut self, k: K) {
        if let Some(first) = self.links.get_mut(0) {
            first.0 = k;
        }
    }

    pub fn insert_smallest_entry(&mut self, e: (K, NonNull<Node<K, V>>)) {
        self.links.insert(0, e);
    }

    pub fn remove_smallest_entry(&mut self) -> (K, NonNull<Node<K, V>>) {
        self.links.remove(0)
    }

    pub fn insert_largest_entry(&mut self, e: (K, NonNull<Node<K, V>>)) {
        self.links.push(e);
    }

    pub fn remove_largest_entry(&mut self) -> (K, NonNull<Node<K, V>>) {
        self.links.pop().unwrap()
    }

    pub fn lmerge_into(&mut self, other: &mut Internal<K, V>) {
        self.links.append(&mut other.links); // TODO: Should just use a VecDeque
        swap(&mut self.links, &mut other.links);
    }

    pub fn rmerge_into(&mut self, other: &mut Internal<K, V>) {
        for (k, node_ptr) in &mut self.links {
            unsafe {
                let node = node_ptr.as_mut();
                node.set_parent(other.parent);
            }
        }

        other.links.append(&mut self.links);

        // self.parent = None // TODO: Maybe not?
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub unsafe fn new_with_children(
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
            child1.set_parent(Some(internal_ptr));
            child2.set_parent(Some(internal_ptr));
        }

        internal_ptr
    }

    pub fn remove(&mut self, k: &K) -> Option<NonNull<Node<K, V>>> {
        let result = self.links.binary_search_by(|(key, _)| key.cmp(k));
        match result {
            Ok(index) => {
                let (k, v) = self.links.remove(index);
                Some(v)
            }
            Err(index) => None,
        }
    }

    pub unsafe fn insert_or_replace(
        &mut self,
        k: K,
        ptr: NonNull<Node<K, V>>,
    ) -> Option<NonNull<Node<K, V>>> {
        let insert = self.links.is_empty();
        let index = self.less_or_equal_to_index(&k);
        if insert {
            self.links.insert(index, (k, ptr));
            None
        } else {
            let out = self.links[index].1;
            self.links[index].1 = ptr;
            Some(out)
        }
    }

    pub fn find_value_less_or_equal_to(&self, k: &K) -> NonNull<Node<K, V>> {
        self.find_entry_less_or_equal_to(k).1
    }

    pub fn find_key_less_or_equal_to(&self, k: &K) -> &K {
        &self.find_entry_less_or_equal_to(k).0
    }

    pub fn find_key_mut_less_or_equal_to(&mut self, k: &K) -> &mut K {
        &mut self.find_entry_mut_less_or_equal_to(k).0
    }

    pub fn find_entry_less_or_equal_to(&self, k: &K) -> &(K, NonNull<Node<K, V>>) {
        debug_assert!(
            !self.links.is_empty(),
            "An internal Node must have children"
        );

        let index = self.less_or_equal_to_index(k);
        &self.links[index]
    }

    pub fn find_entry_mut_less_or_equal_to(&mut self, k: &K) -> &mut (K, NonNull<Node<K, V>>) {
        debug_assert!(
            !self.links.is_empty(),
            "An internal Node must have children"
        );

        let index = self.less_or_equal_to_index(k);
        &mut self.links[index]
    }

    pub fn find_entry(&self, k: &K) -> Option<&(K, NonNull<Node<K, V>>)> {
        debug_assert!(
            !self.links.is_empty(),
            "An internal Node must have children"
        );

        let index = self.links.binary_search_by(|(key, _)| key.cmp(k)).ok()?;

        Some(&self.links[index])
    }

    pub fn find_entry_mut(&mut self, k: &K) -> Option<&mut (K, NonNull<Node<K, V>>)> {
        debug_assert!(
            !self.links.is_empty(),
            "An internal Node must have children"
        );

        let index = self.links.binary_search_by(|(key, _)| key.cmp(k)).ok()?;

        Some(&mut self.links[index])
    }

    pub fn left_index(&self, k: &K) -> Option<usize> {
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

    pub fn left(&self, k: &K) -> Option<&K> {
        let (k, _) = self.left_entry(k)?;
        Some(k)
    }

    pub fn left_mut(&mut self, k: &K) -> Option<&mut K> {
        let (k, _) = self.left_entry_mut(k)?;
        Some(k)
    }

    pub fn left_entry(&self, k: &K) -> Option<&(K, NonNull<Node<K, V>>)> {
        let index = self.left_index(k)?;
        Some(&self.links[index])
    }

    pub fn left_entry_mut(&mut self, k: &K) -> Option<&mut (K, NonNull<Node<K, V>>)> {
        let index = self.left_index(k)?;
        Some(&mut self.links[index])
    }

    pub fn right_index(&self, k: &K) -> Option<usize> {
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

    pub fn right(&self, k: &K) -> Option<&K> {
        let (k, _) = self.right_entry(k)?;
        Some(k)
    }

    pub fn right_mut(&mut self, k: &K) -> Option<&mut K> {
        let (k, _) = self.right_entry_mut(k)?;
        Some(k)
    }

    pub fn right_entry(&self, k: &K) -> Option<&(K, NonNull<Node<K, V>>)> {
        let index = self.right_index(k)?;
        Some(&self.links[index])
    }

    pub fn right_entry_mut(&mut self, k: &K) -> Option<&mut (K, NonNull<Node<K, V>>)> {
        let index = self.right_index(k)?;
        Some(&mut self.links[index])
    }

    pub fn less_or_equal_to_index(&self, k: &K) -> usize {
        self.links
            .binary_search_by(|(key, _)| key.cmp(k))
            .unwrap_or_else(|index| if index == 0 { index } else { index - 1 })
    }

    pub fn parent_raw(&self) -> Option<NonNull<Node<K, V>>> {
        self.parent
    }

    pub fn parent(&self) -> Option<&Internal<K, V>> {
        unsafe { Some(self.parent_raw()?.as_ref().as_internal()) }
    }

    pub fn parent_mut(&mut self) -> Option<&mut Internal<K, V>> {
        unsafe { Some(self.parent_raw()?.as_mut().as_internal_mut()) }
    }
}
