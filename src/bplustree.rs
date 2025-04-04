use std::fmt::Debug;
use std::mem;
use std::mem::swap;
use std::ptr::NonNull;

#[derive(Debug)]
struct Internal<K, V> {
    parent: Option<NonNull<Node<K, V>>>,
    links: Vec<(K, NonNull<Node<K, V>>)>,
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
            .unwrap_or_else(|index| index - 1);

        self.links[index].1
    }
}

#[derive(Debug)]
struct Leaf<K, V> {
    parent: Option<NonNull<Node<K, V>>>,
    data: Vec<(K, V)>,
}

impl<K, V> Leaf<K, V>
where
    K: Ord + Debug,
    V: Ord + Debug,
{
    fn new() -> Self {
        Self {
            parent: None,
            data: vec![],
        }
    }

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        let result = self.data.binary_search_by(|(key, _)| key.cmp(&k));
        let mut pair = (k, v);
        match result {
            Ok(index) => {
                mem::swap(&mut self.data[index], &mut pair);
                Some(pair.1)
            }
            Err(index) => {
                self.data.insert(index, pair);
                None
            }
        }
    }

    fn split(&mut self) -> NonNull<Node<K, V>> {
        let (right) = self.data.split_off(self.data.len() / 2);
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

#[derive(Debug)]
enum Node<K, V> {
    Internal(Internal<K, V>),
    Leaf(Leaf<K, V>),
}

struct BPlusTree<K, V> {
    order: usize,
    root: Option<NonNull<Node<K, V>>>,
}

impl<K, V> BPlusTree<K, V>
where
    K: Ord + Debug,
    V: Ord + Debug,
{
    fn new(order: usize) -> Self {
        Self { order, root: None }
    }

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        if self.root.is_none() {
            let mut leaf = Leaf::new();
            leaf.data.push((k, v));

            let ptr = Box::into_raw(Box::new(Node::Leaf(leaf)));
            self.root = Some(unsafe { NonNull::new_unchecked(ptr) });
            return None;
        }

        let mut ptr = self.find_leaf_node(&k).unwrap(); // SAFETY: We checked that root is not None
        let node = unsafe { ptr.as_mut() };
        let Node::Leaf(leaf) = node else {
            unreachable!();
        };

        let entry = leaf.insert(k, v);
        if leaf.size() > self.max_node_size() {
            let (new) = leaf.split();
            todo!()
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
}

impl<K, V> Drop for BPlusTree<K, V> {
    fn drop(&mut self) {
        let Some(current) = self.root else { return };

        let mut queue = vec![current];
        unsafe {
            while let Some(mut current) = queue.pop() {
                if let Node::Internal(internal) = current.as_mut() {
                    let mut links = vec![];
                    swap(&mut internal.links, &mut links);
                    for (_, ptr) in links {
                        queue.push(ptr);
                    }
                }

                let _ = Box::from_raw(current.as_ptr());
            }
        }
    }
}

fn print_bplustree<K, V>(tree: &BPlusTree<K, V>)
where
    K: Debug,
    V: Debug,
{
    let Some(root) = tree.root else {
        println!("Empty!");
        return;
    };

    unsafe { print_node(root) };
}

unsafe fn print_node<K, V>(ptr: NonNull<Node<K, V>>)
where
    K: Debug,
    V: Debug,
{
    let node = ptr.as_ref();
    match node {
        Node::Internal(internal) => {
            todo!()
        }
        Node::Leaf(leaf) => {
            println!("{:?}", leaf.data);
        }
    }
}

unsafe fn print_ptr<K, V>(ptr: NonNull<Node<K, V>>)
where
    K: Debug,
    V: Debug,
{
    let n = &*ptr.as_ptr();
    println!("({n:p}): {n:?}");
}

#[cfg(test)]
mod tests {
    use crate::bplustree::{Leaf, Node};
    use std::ptr::NonNull;

    mod bplustree {
        use crate::bplustree::{BPlusTree, Node, print_bplustree};

        #[test]
        fn print() {
            let mut btree = BPlusTree::new(4);
            btree.insert((12345, 0), 0);
            btree.insert((12345, 5), 1);
            btree.insert((12345, 10), 2);
            btree.insert((12345, 15), 3);
            print_bplustree(&btree);
        }

        #[test]
        fn insert_single_value() {
            let mut btree = BPlusTree::new(4);
            assert_eq!(btree.insert((12345, 1), 0), None);
        }

        #[test]
        #[ignore = "Not yet implemented"]
        fn insert_multiple_values() {
            let mut btree = BPlusTree::new(4);
            btree.insert((12345, 1), 0);
            btree.insert((12345, 2), 1);
            btree.insert((12345, 15), 2);
            btree.insert((12345, 25), 3);
            btree.insert((12345, 30), 4);
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
        use crate::bplustree::{Leaf, print_node, print_ptr};

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
