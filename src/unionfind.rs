use std::collections::HashMap;
use std::hash::Hash;

/*
  insert(1)
  insert(2)
  insert(4)

  root map
  {
    1: 1
    2: 2
    4: 4
  }

  size map
  {
    1: 1
    2: 1
    4: 1
  }

----------------------

  union(1, 2)

  root map
  {
    1: 1
    2: 1
    4: 4
  }

  size map
  {
    1: 2
    4: 1
  }

----------------------

  union(2, 4)

  root map
  {
    1: 1
    2: 1
    4: 1
  }

  size map
  {
    1: 3
  }




 */
#[derive(Debug)]
struct UnionFind<T> {
    root_map: HashMap<T, T>,
    size_map: HashMap<T, usize>,
}

struct Entry<T> {
    root: T,
    size: usize,
}

impl<T> Entry<T> {
    fn root(&self) -> &T {
        &self.root
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl<T> UnionFind<T>
where
    T: Clone + Eq + Hash,
{
    fn new() -> Self {
        Self {
            root_map: HashMap::new(),
            size_map: HashMap::new(),
        }
    }

    fn insert(&mut self, elem: T) -> T {
        let root = self.root_map.entry(elem.clone()).or_insert(elem).clone();
        self.size_map.entry(root.clone()).or_insert(1);

        root
    }

    fn get(&self, elem: &T) -> Option<Entry<T>> {
        let root = self.root_map.get(elem)?.clone();
        let size = *self.size_map.get(elem)?;
        let entry = Entry { root, size };
        Some(entry)
    }

    /*

                         -----
                         v   |
     0 -> 1 -> 2 -> 3 -> 4 --^

     find(0) -> 1
     find(1) -> 2
     find(2) -> 3
     find(3) -> 4
     find(4) -> 4

    */
    fn find<'a>(&'a self, elem: &'a T) -> Option<&'a T> {
        let mut current = elem;
        loop {
            let root = self.root_map.get(current)?;
            if current == root {
                break;
            }

            current = root;
        }

        Some(current)
    }

    fn union(&mut self, elem1: T, elem2: T) -> T {
        if self.connected(&elem1, &elem2) {
            return self.find(&elem1).unwrap().clone();
        }

        if self.root_map.get(&elem1).is_none() {
            self.insert(elem1.clone());
        }

        if self.root_map.get(&elem2).is_none() {
            self.insert(elem2.clone());
        }

        // unify
        let root1 = self.find(&elem1).unwrap().clone();
        let root2 = self.find(&elem2).unwrap().clone();
        let size1 = *self.size_map.get(&root1).unwrap();
        let size2 = *self.size_map.get(&root2).unwrap();

        let root;
        if size1 >= size2 {
            let size1 = self.size_map.get_mut(&root1).unwrap();
            *size1 += size2;

            self.size_map.remove(&root2);
            let root2 = self.root_map.get_mut(&root2).unwrap();
            *root2 = root1.clone();
            root = root1;
        } else {
            let size2 = self.size_map.get_mut(&root2).unwrap();
            *size2 += size1;

            self.size_map.remove(&root1);
            let root1 = self.root_map.get_mut(&root1).unwrap();
            *root1 = root2.clone();
            root = root2;
        }

        root
    }

    fn size(&self) -> usize {
        // TODO: Bad.
        let mut output = 0;
        for size in self.size_map.values() {
            output += size;
        }
        output
    }

    fn sets(&self) -> usize {
        self.size_map.keys().len()
    }

    fn connected(&self, elem1: &T, elem2: &T) -> bool {
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
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;

    #[test]
    fn basic_1() {
        let mut uf: UnionFind<i32> = UnionFind::new();

        assert_eq!(uf.size(), 0);
        assert_eq!(uf.sets(), 0);

        uf.insert(10);
        assert_eq!(uf.size(), 1);
        assert_eq!(uf.sets(), 1);

        assert_eq!(uf.find(&10), Some(&10));

        assert_eq!(uf.union(10, 12), 10);
        assert_eq!(uf.size(), 2);
        assert_eq!(uf.sets(), 1);

        assert_eq!(uf.find(&12), Some(&10));
    }

    #[test]
    fn insert_1() {
        let mut uf = UnionFind::new();

        assert_eq!(uf.insert(1), 1);
        assert_eq!(uf.insert(2), 2);
        assert_eq!(uf.insert(1), 1);
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

    #[test]
    fn union_1() {
        let mut uf = UnionFind::new();
        uf.union(1, 2);

        assert_eq!(uf.root_map, HashMap::from([(1, 1), (2, 1)]));
        assert_eq!(uf.size_map, HashMap::from([(1, 2)]));

        uf.union(3, 4);
        assert_eq!(uf.root_map, HashMap::from([(1, 1), (2, 1), (3, 3), (4, 3)]));
        assert_eq!(uf.size_map, HashMap::from([(1, 2), (3, 2)]));

        uf.union(2, 4);
        assert_eq!(uf.root_map, HashMap::from([(1, 1), (2, 1), (3, 1), (4, 3)]));
        assert_eq!(uf.size_map, HashMap::from([(1, 4)]));
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

    #[test]
    fn test() {
        let mut hm: HashMap<i32, usize> = HashMap::new();
        let m = hm.insert(10, 50);
    }
}
