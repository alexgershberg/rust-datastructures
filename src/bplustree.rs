#![allow(unused)]

const ORDER: usize = 4;

struct Internal<K, V> {
    links: Vec<Node<K, V>>,
}

struct Leaf<K, V> {
    data: Vec<(K, V)>,
}

enum Node<K, V> {
    Internal(Internal<K, V>),
    Leaf(Leaf<K, V>),
}

struct BPlusTree<K, V> {
    root: Node<K, V>,
}

impl<K, V> BPlusTree<K, V> {
    fn new() -> Self {
        todo!()
    }

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        todo!()
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
}

#[cfg(test)]
mod tests {
    use crate::bplustree::BPlusTree;

    #[test]
    fn insert_1() {
        let mut btree = BPlusTree::new();
        btree.insert((12345, 1), 0);
        btree.insert((12345, 2), 1);
        btree.insert((12345, 15), 2);
        btree.insert((12345, 25), 3);
        btree.insert((12345, 30), 4);
    }
}
