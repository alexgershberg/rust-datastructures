use std::cell::{Cell, Ref, RefCell};
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Debug)]
pub struct UnionFind<T> {
    root_map: RefCell<HashMap<Rc<T>, Rc<T>>>,
    size_map: RefCell<HashMap<Rc<T>, usize>>,
}

pub struct Entry<T> {
    root: T,
    size: usize,
}

impl<T> Entry<Rc<T>> {
    pub fn root(&self) -> &T {
        &self.root
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<T> UnionFind<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            root_map: RefCell::new(HashMap::new()),
            size_map: RefCell::new(HashMap::new()),
        }
    }

    pub fn insert(&mut self, elem: T) {
        let rc = Rc::new(elem);
        let root = self
            .root_map
            .borrow_mut()
            .entry(rc.clone())
            .or_insert(rc.clone())
            .clone();
        self.size_map.borrow_mut().entry(root).or_insert(1);
    }

    pub fn get(&self, elem: &T) -> Option<Entry<Rc<T>>> {
        let root = self.root_map.borrow().get(elem)?.clone();
        let size = *self.size_map.borrow().get(elem)?;
        let entry = Entry { root, size };
        Some(entry)
    }

    pub fn find(&self, elem: &T) -> Option<Rc<T>> {
        let mut current = Rc::new(elem.clone());
        loop {
            let root_map = self.root_map.borrow();
            let root = root_map.get(&*current).cloned()?;
            if *current == *root {
                break;
            }

            current = root;
        }

        Some(current)
    }

    pub fn union(&mut self, elem1: T, elem2: T) -> T {
        if self.connected(&elem1, &elem2) {
            return (*self.find(&elem1).unwrap()).clone();
        }

        if self.root_map.borrow().get(&elem1).is_none() {
            self.insert(elem1.clone());
        }

        if self.root_map.borrow().get(&elem2).is_none() {
            self.insert(elem2.clone());
        }

        // unify
        let binding = self.find(&elem1).unwrap();
        let root1 = binding.as_ref();

        let binding = self.find(&elem2).unwrap();
        let root2 = binding.as_ref();

        let size1 = *self.size_map.borrow().get(root1).unwrap();
        let size2 = *self.size_map.borrow().get(root2).unwrap();

        let root = if size1 >= size2 {
            {
                let mut borrow = self.size_map.borrow_mut();
                let size1 = borrow.get_mut(root1).unwrap();
                *size1 += size2;
            }
            self.size_map.borrow_mut().remove(root2);

            let mut borrow = self.root_map.borrow_mut();
            let root2 = borrow.get_mut(root2).unwrap();
            *root2 = Rc::new(root1.clone());

            root1
        } else {
            {
                let mut borrow = self.size_map.borrow_mut();
                let size2 = borrow.get_mut(root2).unwrap();
                *size2 += size1;
            }
            self.size_map.borrow_mut().remove(root1);

            let mut borrow = self.root_map.borrow_mut();
            let root1 = borrow.get_mut(root1).unwrap();
            *root1 = Rc::new(root2.clone());

            root2
        };

        root.clone()
    }

    pub fn size(&self) -> usize {
        // TODO: Bad.
        let mut output = 0;
        for size in self.size_map.borrow().values() {
            output += size;
        }
        output
    }

    pub fn sets(&self) -> usize {
        self.size_map.borrow().keys().len()
    }

    pub fn connected(&self, elem1: &T, elem2: &T) -> bool {
        let root1 = self.find(elem1);
        if root1.is_none() {
            return false;
        }

        let root2 = self.find(elem2);
        if root2.is_none() {
            return false;
        }

        root1 == root2
    }
}

#[cfg(test)]
mod tests {
    use crate::unionfind::UnionFind;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn basic_1() {
        let mut uf: UnionFind<i32> = UnionFind::new();

        assert_eq!(uf.size(), 0);
        assert_eq!(uf.sets(), 0);

        uf.insert(10);
        assert_eq!(uf.size(), 1);
        assert_eq!(uf.sets(), 1);

        assert_eq!(uf.find(&10), Some(Rc::new(10)));

        assert_eq!(uf.union(10, 12), 10);
        assert_eq!(uf.size(), 2);
        assert_eq!(uf.sets(), 1);

        assert_eq!(uf.find(&12), Some(Rc::new(10)));
    }

    #[test]
    fn insert_1() {
        let mut uf = UnionFind::new();

        uf.insert(1);
        uf.insert(2);
        uf.insert(1);

        assert_eq!(uf.size(), 2);
        assert_eq!(uf.sets(), 2);
    }

    #[test]
    fn get_1() {
        let mut uf = UnionFind::new();
        uf.insert(5);

        let entry = uf.get(&5).unwrap();
        assert_eq!(entry.root(), &5);
        assert_eq!(entry.size(), 1);
    }

    #[test]
    fn connected_1() {
        let mut uf = UnionFind::new();

        uf.insert(1);
        uf.insert(2);
        assert!(!uf.connected(&1, &2));

        uf.union(1, 2);
        assert!(uf.connected(&1, &2));
    }

    fn root_entries(uf: &UnionFind<i32>) -> Vec<(i32, i32)> {
        let mut v = uf
            .root_map
            .borrow()
            .iter()
            .map(|(key, value)| (**key, **value))
            .collect::<Vec<(i32, i32)>>();
        v.sort();
        v
    }

    fn size_entries(uf: &UnionFind<i32>) -> Vec<(i32, usize)> {
        let mut v = uf
            .size_map
            .borrow()
            .iter()
            .map(|(key, value)| (**key, *value))
            .collect::<Vec<_>>();
        v.sort();
        v
    }

    #[test]
    fn union_1() {
        let mut uf = UnionFind::new();

        uf.union(1, 2);

        let root_entries = crate::unionfind::tests::root_entries(&uf);
        assert_eq!(root_entries[0], (1, 1));
        assert_eq!(root_entries[1], (2, 1));
        assert_eq!(root_entries.len(), 2);
        let size_entries = crate::unionfind::tests::size_entries(&uf);
        assert_eq!(size_entries[0], (1, 2));
        assert_eq!(size_entries.len(), 1);

        uf.union(3, 4);

        let root_entries = crate::unionfind::tests::root_entries(&uf);
        assert_eq!(root_entries[0], (1, 1));
        assert_eq!(root_entries[1], (2, 1));
        assert_eq!(root_entries[2], (3, 3));
        assert_eq!(root_entries[3], (4, 3));
        assert_eq!(root_entries.len(), 4);
        let size_entries = crate::unionfind::tests::size_entries(&uf);
        assert_eq!(size_entries[0], (1, 2));
        assert_eq!(size_entries[1], (3, 2));
        assert_eq!(size_entries.len(), 2);

        uf.union(2, 4);

        let root_entries = crate::unionfind::tests::root_entries(&uf);
        assert_eq!(root_entries[0], (1, 1));
        assert_eq!(root_entries[1], (2, 1));
        assert_eq!(root_entries[2], (3, 1));
        assert_eq!(root_entries[3], (4, 3));
        assert_eq!(root_entries.len(), 4);
        let size_entries = crate::unionfind::tests::size_entries(&uf);
        assert_eq!(size_entries[0], (1, 4));
        assert_eq!(size_entries.len(), 1);
    }

    #[test]
    fn find_1() {
        let uf = UnionFind::new();
        assert_eq!(uf.find(&1), None);
    }

    #[test]
    fn sets_1() {
        let mut uf = UnionFind::new();
        uf.union(0, 1);
        uf.union(2, 3);
        uf.union(1, 3);
        assert_eq!(uf.sets(), 1);
    }

    #[test]
    fn sets_2() {
        let mut uf = UnionFind::new();
        uf.union(0, 1);
        uf.union(2, 3);
        uf.union(1, 3);
        assert_eq!(uf.sets(), 1);

        uf.union(4, 5);
        uf.union(6, 7);
        uf.union(5, 7);
        assert_eq!(uf.sets(), 2);

        uf.union(3, 7);
        assert_eq!(uf.sets(), 1);
    }

    #[test]
    fn uf_with_strings() {
        let mut uf = UnionFind::new();
        uf.union("A", "B");
        uf.union("C", "D");

        uf.union("F", "G");
        uf.union("H", "I");
        assert_eq!(uf.sets(), 4);
        assert_eq!(uf.size(), 8);
    }
}
