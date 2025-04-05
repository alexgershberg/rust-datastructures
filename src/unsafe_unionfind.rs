use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Debug;
use std::hash::Hash;
use std::ptr::NonNull;

#[derive(Debug, Clone)]
pub struct UnionFind<T> {
    roots: HashMap<NonNull<T>, NonNull<T>>,
    sizes: HashMap<NonNull<T>, usize>,
    values: HashMap<T, NonNull<T>>,
    size: usize,
}

unsafe impl<T: Send> Send for UnionFind<T> {}
unsafe impl<T: Send> Sync for UnionFind<T> {}

impl<T> Drop for UnionFind<T> {
    fn drop(&mut self) {
        for key in self.roots.keys() {
            unsafe {
                let _ = Box::from_raw(key.as_ptr());
            };
        }
    }
}

impl<T> UnionFind<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            roots: HashMap::new(),
            sizes: HashMap::new(),
            values: HashMap::new(),
            size: 0,
        }
    }

    pub fn union(&mut self, elem1: T, elem2: T) -> &T {
        if self.connected(&elem1, &elem2) {
            return self.find(&elem1).unwrap();
        }

        let elem1 = elem1.clone();
        let elem2 = elem2.clone();

        let ptr1 = match self.internal_get(&elem1) {
            None => self.internal_insert(elem1),
            Some(ptr) => ptr,
        };

        let ptr2 = match self.internal_get(&elem2) {
            None => self.internal_insert(elem2),
            Some(ptr) => ptr,
        };

        let ptr1 = self.internal_find(ptr1).unwrap();
        let ptr2 = self.internal_find(ptr2).unwrap();

        let size1 = *self.sizes.get(&ptr1).unwrap();
        let size2 = *self.sizes.get(&ptr2).unwrap();

        let root = if size1 >= size2 {
            self.internal_union(ptr1, ptr2)
        } else {
            self.internal_union(ptr2, ptr1)
        };

        unsafe { root.as_ref() }
    }
    pub fn find(&self, elem: &T) -> Option<&T> {
        let ptr = self.values.get(elem).cloned()?;
        let root = self.internal_find(ptr)?;

        unsafe { Some(root.as_ref()) }
    }

    pub fn insert(&mut self, elem: T) {
        let _ = self.internal_insert(elem);
    }

    pub fn connected(&self, elem1: &T, elem2: &T) -> bool {
        let Some(root1) = self.find(elem1) else {
            return false;
        };

        let Some(root2) = self.find(elem2) else {
            return false;
        };

        root1 == root2
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn sets(&self) -> usize {
        self.sizes.keys().len()
    }

    pub fn contains(&self, elem: &T) -> bool {
        self.internal_get(elem).is_some()
    }

    fn internal_insert(&mut self, elem: T) -> NonNull<T> {
        let ptr = match self.values.entry(elem.clone()) {
            Entry::Occupied(occupied) => return *occupied.get(),
            Entry::Vacant(vacant) => {
                let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(elem))) };
                vacant.insert(ptr);
                ptr
            }
        };

        let root = *self.roots.entry(ptr).or_insert(ptr);
        if let Entry::Vacant(vacant) = self.sizes.entry(root) {
            vacant.insert(1);
            self.size += 1;
        };

        ptr
    }

    fn internal_get(&self, elem: &T) -> Option<NonNull<T>> {
        self.values.get(elem).cloned()
    }

    fn internal_union(&mut self, root1: NonNull<T>, root2: NonNull<T>) -> NonNull<T> {
        // merge root2 into root1

        let rt2 = self.roots.get_mut(&root2).unwrap();
        *rt2 = root1;

        let size2 = *self.sizes.get(&root2).unwrap();
        let size1 = self.sizes.get_mut(&root1).unwrap();
        *size1 += size2;

        self.sizes.remove(&root2);

        root1
    }

    fn internal_find(&self, ptr: NonNull<T>) -> Option<NonNull<T>> {
        let mut current = ptr;
        loop {
            let root = self.roots.get(&current).cloned()?;
            if current == root {
                break;
            }

            current = root;
        }

        Some(current)
    }
}

unsafe fn print_entry<T: Debug>(entry: NonNull<T>) {
    print!("\t({:p}) {:?}", entry, unsafe { &*(entry.as_ptr()) });
}

fn pretty_print<T: Debug>(uf: &UnionFind<T>) {
    println!("roots {{");
    for (&key, &value) in &uf.roots {
        unsafe {
            print_entry(key);
            print!(": ");
            print_entry(value);
        };
        println!()
    }
    println!("\n}}");

    println!("sizes {{");
    for (&key, &value) in &uf.sizes {
        unsafe { print_entry(key) };
        print!(": ");
        print!("{}", value);
        println!()
    }
    println!("\n}}");

    println!("values {{");
    for (key, &value) in &uf.values {
        print!("\t{key:?}");
        print!(": ");
        unsafe { print_entry(value) };
        println!()
    }
    println!("\n}}");

    println!("size: {}", uf.size);
}

#[cfg(test)]
mod tests {
    use crate::unsafe_unionfind::{UnionFind, pretty_print};
    use std::sync::{Arc, Mutex};
    use std::thread;
    

    #[test]
    fn pretty_printing() {
        let mut uf = UnionFind::new();
        uf.insert("Hello!");
        pretty_print(&uf);
    }

    #[test]
    fn basic_1() {
        let mut uf: UnionFind<i32> = UnionFind::new();

        assert_eq!(uf.size(), 0);
        assert_eq!(uf.sets(), 0);

        uf.insert(10);
        assert_eq!(uf.size(), 1);
        assert_eq!(uf.sets(), 1);

        assert_eq!(uf.find(&10), Some(&10));

        assert_eq!(uf.union(10, 12), &10);
        assert_eq!(uf.size(), 2);
        assert_eq!(uf.sets(), 1);

        assert_eq!(uf.find(&12), Some(&10));
    }

    #[test]
    fn union_1() {
        let mut uf = UnionFind::new();
        assert_eq!(uf.union(0, 1), &0);
        assert_eq!(uf.sets(), 1);
        assert_eq!(uf.size(), 2);
    }

    #[test]
    fn union_2() {
        let mut uf = UnionFind::new();
        assert_eq!(uf.union(0, 1), &0);
        assert_eq!(uf.union(2, 3), &2);
        assert_eq!(uf.union(0, 2), &0);

        assert_eq!(uf.union(4, 5), &4);
        assert_eq!(uf.union(6, 7), &6);
        assert_eq!(uf.union(4, 6), &4);

        assert_eq!(uf.union(0, 4), &0);
    }

    #[test]
    fn find_1() {
        let mut uf = UnionFind::new();
        uf.insert(0);
        assert_eq!(uf.find(&0), Some(&0));
    }

    #[test]
    fn connected_1() {
        let mut uf = UnionFind::new();
        uf.insert(0);
        uf.insert(1);
        assert!(!uf.connected(&0, &1));

        uf.union(1, 2);
        assert!(uf.connected(&1, &2));
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

    #[test]
    fn threads_1() {
        let uf = Arc::new(Mutex::new(UnionFind::new()));
        let uf1 = uf.clone();
        let t1 = thread::spawn(move || {
            let mut uf = uf1.lock().unwrap();
            for i in 0..3 {
                uf.insert(i);
            }
        });

        let uf2 = uf.clone();
        let t2 = thread::spawn(move || {
            let mut uf = uf2.lock().unwrap();
            for i in 3..6 {
                uf.insert(i);
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let mut uf = uf.lock().unwrap();

        for i in 0..5 {
            uf.union(i, i + 1);
        }

        pretty_print(&uf);
    }

    #[test]
    fn threads_2() {
        let mut uf = UnionFind::new();
        for i in 0..5 {
            uf.insert(i);
        }
        let ptr = Box::into_raw(Box::new(uf));
        let r1 = unsafe { ptr.as_ref().unwrap() };
        let r2 = unsafe { ptr.as_ref().unwrap() };

        let t1 = thread::spawn(move || {
            for i in 0..5 {
                let f = r1.find(&i);
                println!("[1] {f:?}");
            }
        });

        let t2 = thread::spawn(move || {
            for i in 0..5 {
                let f = r2.find(&i);
                println!("[2] {f:?}");
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();

        let _ = unsafe { Box::from_raw(ptr) };
    }

    #[test]
    fn threads_3() {
        let mut uf = UnionFind::new();
        for i in 0..5 {
            uf.insert(i);
        }

        thread::scope(|s| {
            s.spawn(|| {
                for i in 0..5 {
                    let f = uf.find(&i);
                    println!("[1] {f:?}");
                }
            });
            s.spawn(|| {
                for i in 0..5 {
                    let f = uf.find(&i);
                    println!("[2] {f:?}");
                }
            });
        });
    }
}
