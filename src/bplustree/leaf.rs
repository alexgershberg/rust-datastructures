use crate::bplustree::internal::Internal;
use crate::bplustree::node::Node;
use std::fmt::Debug;
use std::mem::swap;
use std::ptr::NonNull;

#[derive(Debug)]
pub(crate) struct Leaf<K, V> {
    pub(crate) parent: Option<NonNull<Node<K, V>>>,
    pub(crate) data: Vec<(K, V)>,
}

impl<K, V> Leaf<K, V>
where
    K: Ord + PartialOrd + Clone,
{
    pub(crate) fn new() -> Self {
        Self {
            parent: None,
            data: vec![],
        }
    }

    pub(crate) fn split(&mut self) -> NonNull<Node<K, V>> {
        let right = self.data.split_off(self.data.len() / 2);
        debug_assert!(self.data.len() <= right.len());

        unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node::Leaf(Leaf {
                parent: None,
                data: right,
            }))))
        }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn keys(&self) -> Vec<&K> {
        self.data.iter().map(|(k, _)| k).collect::<Vec<_>>()
    }

    fn smallest_entry(&self) -> &(K, V) {
        self.data.first().unwrap()
    }

    pub fn smallest_key(&self) -> &K {
        &self.smallest_entry().0
    }

    pub fn insert_smallest_entry(&mut self, e: (K, V)) {
        self.data.insert(0, e);
    }

    pub fn remove_smallest_entry(&mut self) -> (K, V) {
        self.data.remove(0)
    }

    pub fn insert_largest_entry(&mut self, e: (K, V)) {
        self.data.push(e);
    }

    pub fn remove_largest_entry(&mut self) -> (K, V) {
        self.data.pop().unwrap()
    }

    pub fn lmerge_into(&mut self, other: &mut Leaf<K, V>) {
        self.data.append(&mut other.data); // TODO: Should just use a VecDeque
        swap(&mut self.data, &mut other.data);
    }

    pub fn rmerge_into(&mut self, other: &mut Leaf<K, V>) {
        other.data.append(&mut self.data);
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
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

    pub fn remove(&mut self, k: &K) -> Option<V> {
        let result = self.data.binary_search_by(|(key, _)| key.cmp(k));
        match result {
            Ok(index) => {
                let (k, v) = self.data.remove(index);
                Some(v)
            }
            Err(index) => None,
        }
    }

    pub fn find(&self, k: &K) -> Option<&(K, V)> {
        self.data.iter().find(|(key, _)| key == k)
    }

    pub fn find_mut(&mut self, k: &K) -> Option<&mut (K, V)> {
        self.data.iter_mut().find(|(key, _)| key == k)
    }

    pub fn update_parent_smallest_key(&mut self) {
        let current_smallest_key = self.smallest_key().clone();
        let mut current_parent = self.parent_mut();
        while let Some(parent) = current_parent {
            let needs_updating = *parent.smallest_key() > current_smallest_key;
            if needs_updating {
                let k = parent.find_key_mut_less_or_equal_to(&current_smallest_key);
                *k = current_smallest_key.clone();
            } else {
                break;
            }

            current_parent = parent.parent_mut();
        }
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
