use crate::bplustree::internal::Internal;
use crate::bplustree::leaf::Leaf;
use std::fmt::Debug;
use std::ptr::NonNull;

#[derive(Debug)]
pub enum NodeEntry<K, V> {
    Internal((K, NonNull<Node<K, V>>)),
    Leaf((K, V)),
}

impl<K, V> NodeEntry<K, V> {
    pub fn new(k: K, v: NodeValue<K, V>) -> Self {
        match v {
            NodeValue::Internal(v) => Self::Internal((k, v)),
            NodeValue::Leaf(v) => Self::Leaf((k, v)),
        }
    }

    pub fn key(&self) -> &K {
        match self {
            NodeEntry::Internal((k, _)) => k,
            NodeEntry::Leaf((k, _)) => k,
        }
    }
}

#[derive(Debug)]
pub enum NodeValue<K, V> {
    Leaf(V),
    Internal(NonNull<Node<K, V>>),
}

#[derive(Debug)]
pub enum Node<K, V> {
    Internal(Internal<K, V>),
    Leaf(Leaf<K, V>),
}

impl<K, V> Node<K, V>
where
    K: Ord + PartialOrd + Clone + Debug,
    V: Ord + PartialOrd + Clone + Debug,
{
    pub fn parent_raw(&self) -> Option<NonNull<Node<K, V>>> {
        match self {
            Node::Internal(internal) => internal.parent_raw(),
            Node::Leaf(leaf) => leaf.parent_raw(),
        }
    }

    pub fn parent(&self) -> Option<&Internal<K, V>> {
        unsafe { Some(self.parent_raw()?.as_ref().as_internal()) }
    }

    pub fn parent_mut(&mut self) -> Option<&mut Internal<K, V>> {
        unsafe { Some(self.parent_raw()?.as_mut().as_internal_mut()) }
    }

    /// SAFETY:
    ///  * ptr MUST NOT point to self
    ///  * ptr MUST NOT be dangling
    pub unsafe fn set_parent(&mut self, parent: Option<NonNull<Node<K, V>>>) {
        match self {
            Node::Internal(internal) => internal.parent = parent,
            Node::Leaf(leaf) => leaf.parent = parent,
        }
    }

    pub fn keys(&self) -> Vec<&K> {
        match self {
            Node::Internal(internal) => internal.keys(),
            Node::Leaf(leaf) => leaf.keys(),
        }
    }

    pub fn smallest_key(&self) -> &K {
        match self {
            Node::Internal(internal) => internal.smallest_key(),
            Node::Leaf(leaf) => leaf.smallest_key(),
        }
    }

    pub(crate) fn smallest_value(&self) -> NodeValue<K, V> {
        match self {
            Node::Internal(internal) => NodeValue::Internal(internal.smallest_value()),
            Node::Leaf(leaf) => NodeValue::Leaf(leaf.smallest_value()),
        }
    }

    pub(crate) fn largest_key(&self) -> Option<&K> {
        match self {
            Node::Internal(internal) => internal.links.last().map(|(k, _)| k),
            Node::Leaf(leaf) => leaf.data.last().map(|(k, _)| k),
        }
    }

    fn set_smallest(&mut self, k: K) {
        match self {
            Node::Internal(internal) => {
                internal.set_smallest(k);
            }
            Node::Leaf(leaf) => {
                if let Some(first) = leaf.data.get_mut(0) {
                    first.0 = k;
                }
            }
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Node::Internal(internal) => internal.size(),
            Node::Leaf(leaf) => leaf.size(),
        }
    }

    pub fn as_internal(&self) -> &Internal<K, V> {
        match self {
            Node::Internal(internal) => internal,
            Node::Leaf(_leaf) => {
                panic!("Expected an Internal node but got Leaf")
            }
        }
    }

    pub fn as_internal_mut(&mut self) -> &mut Internal<K, V> {
        match self {
            Node::Internal(internal) => internal,
            Node::Leaf(_leaf) => {
                panic!("Expected an Internal node but got Leaf")
            }
        }
    }

    pub fn as_leaf(&self) -> &Leaf<K, V> {
        match self {
            Node::Internal(_internal) => {
                panic!("Expected a Leaf node but got Internal")
            }
            Node::Leaf(leaf) => leaf,
        }
    }

    pub(crate) fn as_leaf_mut(&mut self) -> &mut Leaf<K, V> {
        match self {
            Node::Internal(_internal) => {
                panic!("Expected a Leaf node but got Internal")
            }
            Node::Leaf(leaf) => leaf,
        }
    }

    pub fn insert_smallest_entry(&mut self, e: NodeEntry<K, V>) {
        match (self, e) {
            (Node::Internal(internal), NodeEntry::Internal(e)) => internal.insert_smallest_entry(e),
            (Node::Leaf(leaf), NodeEntry::Leaf(e)) => leaf.insert_smallest_entry(e),
            (Node::Leaf(..), NodeEntry::Internal(..)) => {
                panic!("Trying to insert Internal node entry into a Leaf!")
            }
            (Node::Internal(..), NodeEntry::Leaf(..)) => {
                panic!("Trying to insert Leaf entry into an Internal node!")
            }
        }
    }

    pub fn remove_smallest_entry(&mut self) -> NodeEntry<K, V> {
        match self {
            Node::Internal(internal) => NodeEntry::Internal(internal.remove_smallest_entry()),
            Node::Leaf(leaf) => NodeEntry::Leaf(leaf.remove_smallest_entry()),
        }
    }

    pub fn insert_largest_entry(&mut self, e: NodeEntry<K, V>) {
        match (self, e) {
            (Node::Internal(internal), NodeEntry::Internal(e)) => internal.insert_largest_entry(e),
            (Node::Leaf(leaf), NodeEntry::Leaf(e)) => leaf.insert_largest_entry(e),
            (Node::Leaf(..), NodeEntry::Internal(..)) => {
                panic!("Trying to insert Internal node entry into a Leaf!")
            }
            (Node::Internal(..), NodeEntry::Leaf(..)) => {
                panic!("Trying to insert Leaf entry into an Internal node!")
            }
        }
    }

    pub fn remove_largest_entry(&mut self) -> NodeEntry<K, V> {
        match self {
            Node::Internal(internal) => NodeEntry::Internal(internal.remove_largest_entry()),
            Node::Leaf(leaf) => NodeEntry::Leaf(leaf.remove_largest_entry()),
        }
    }

    pub fn lmerge_into(&mut self, other: &mut Node<K, V>) {
        match self {
            Node::Internal(internal) => internal.lmerge_into(other.as_internal_mut()),
            Node::Leaf(leaf) => leaf.lmerge_into(other.as_leaf_mut()),
        }
    }

    pub fn rmerge_into(&mut self, other: &mut Node<K, V>) {
        match self {
            Node::Internal(internal) => internal.rmerge_into(other.as_internal_mut()),
            Node::Leaf(leaf) => leaf.rmerge_into(other.as_leaf_mut()),
        }
    }

    pub fn is_root(&self) -> bool {
        match self {
            Node::Internal(internal) => internal.is_root(),
            Node::Leaf(leaf) => leaf.is_root(),
        }
    }

    pub fn update_key_from_smaller_to_bigger(&mut self, k: K) {
        match self {
            Node::Internal(internal) => {
                todo!()
                // let entry = internal.find_mut(&k);
                // (*entry).0 = k;
            }
            Node::Leaf(leaf) => {
                todo!()
            }
        }
    }

    pub fn update_key_from_bigger_to_smaller(&mut self, k: K) {
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

    pub fn left(&self, k: &K) -> Option<&K> {
        match self {
            Node::Internal(internal) => internal.left(k),
            Node::Leaf(leaf) => {
                todo!()
            }
        }
    }

    pub fn right(&self, k: &K) -> Option<&K> {
        match self {
            Node::Internal(internal) => internal.right(k),
            Node::Leaf(leaf) => {
                todo!()
            }
        }
    }

    pub(crate) fn remove_key(&mut self, k: &K) -> Option<NodeValue<K, V>> {
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

    unsafe fn insert(&mut self, k: K, v: NodeValue<K, V>) {
        match (self, v) {
            (Node::Internal(internal), NodeValue::Internal(v)) => {
                if let Some(old) = internal.insert_or_replace(k, v) {
                    println!("insert or replace yielded a ptr so we're freeing it: {old:?}");
                    let _ = unsafe { Box::from_raw(old.as_ptr()) };
                }
            }
            (Node::Leaf(leaf), NodeValue::Leaf(v)) => todo!(),
            (Node::Leaf(..), NodeValue::Internal(..)) => {
                panic!("Trying to insert Internal node value into a Leaf!")
            }
            (Node::Internal(..), NodeValue::Leaf(..)) => {
                panic!("Trying to insert Leaf value into an Internal node!")
            }
        }
    }
}
