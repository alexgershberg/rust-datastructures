use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::{Debug, Formatter, Write};
use std::ptr::{NonNull, write};

struct Node {
    children: HashMap<char, NonNull<Node>>,
    terminal: bool,
}

impl Node {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            terminal: false,
        }
    }

    fn new_non_null() -> NonNull<Self> {
        unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Node::new()))) }
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let children_debug_output = self
            .children
            .iter()
            .map(|(c, ptr)| {
                let node = unsafe { &*ptr.as_ptr() };
                (c, (ptr, node))
            })
            .collect::<HashMap<_, _>>();

        f.debug_struct("Node")
            .field("children", &children_debug_output)
            .field("terminal", &self.terminal)
            .finish()
    }
}

pub struct Trie {
    root: Option<NonNull<Node>>,
}

impl Trie {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, text: &str) -> bool {
        if self.root.is_none() {
            self.root = Some(Node::new_non_null());
        }

        let mut current = self.root.unwrap();
        for c in text.chars() {
            let children = unsafe { &mut (*current.as_ptr()).children };

            let entry = children.entry(c);
            match entry {
                Entry::Occupied(occupied) => {
                    current = occupied.get().clone();
                }
                Entry::Vacant(vacant) => {
                    let new = Node::new_non_null();
                    vacant.insert(new);
                    current = new;
                }
            }
        }

        let current = unsafe { current.as_mut() };
        if current.terminal {
            false
        } else {
            current.terminal = true;
            true
        }
    }

    pub fn remove(&mut self, text: &str) -> bool {
        todo!()
    }

    pub fn contains(&self, text: &str) -> bool {
        let Some(root) = self.root else {
            return false;
        };

        let mut current = root;
        for c in text.chars() {
            let node = unsafe { &*current.as_ptr() };
            let children = &node.children;
            let Some(&node) = children.get(&c) else {
                return false;
            };
            current = node;
        }

        let current = unsafe { &*current.as_ptr() };
        current.terminal
    }
}

impl Debug for Trie {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("Trie");

        if let Some(root) = &self.root {
            let tup = (root, unsafe { &*root.as_ptr() });
            debug_struct.field("root", &tup);
        } else {
            debug_struct.field("root", &self.root);
        }

        debug_struct.finish()
    }
}

impl Drop for Trie {
    fn drop(&mut self) {
        let Some(root) = self.root.take() else { return };

        let mut stack = vec![root];

        while let Some(ptr) = stack.pop() {
            let node = unsafe { ptr.as_ref() };
            let children = &node.children;

            for child in children.values() {
                stack.push(child.clone());
            }

            let b = unsafe { Box::from_raw(ptr.as_ptr()) };
        }
    }
}

fn fmt_ptr<T>(ptr: NonNull<T>, f: &mut Formatter<'_>) -> std::fmt::Result
where
    T: Debug,
{
    f.debug_map()
        .entry(&ptr, unsafe { &*ptr.as_ptr() })
        .finish()
}

fn print_node(node: &Node) {
    let debug = node
        .children
        .iter()
        .map(|(c, ptr)| {
            let node = unsafe { &*ptr.as_ptr() };
            print_node(node);
            c
        })
        .collect::<Vec<_>>();
    println!("{:?}", debug);
}

fn print_trie(trie: &Trie) {
    println!("root: {:?}", trie.root);
    if let Some(root) = trie.root {
        let node = unsafe { root.as_ref() };
        for (char, &ptr) in &node.children {
            println!("{char}: ({ptr:?}: {:#?})", unsafe { &*(ptr.as_ptr()) });
        }
        println!("terminal: {}", node.terminal)
    }
}

#[cfg(test)]
mod tests {
    use crate::trie::{Node, Trie, print_trie};
    use std::collections::HashMap;
    use std::ptr::NonNull;

    #[test]
    fn debug_node() {
        let node = Node {
            children: HashMap::new(),
            terminal: true,
        };
        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(node))) };
        let node = Node {
            children: HashMap::from([('C', ptr)]),
            terminal: false,
        };
        println!("{node:#?}");
    }

    #[test]
    fn debug_trie() {
        let mut trie = Trie::new();
        trie.insert("Alex");
        println!("{:#?}", &trie);
    }

    #[test]
    fn insert_1() {
        let mut trie = Trie::new();
        assert!(trie.insert("Hello"));
        assert!(trie.insert("Stop"));
        assert!(trie.insert("Help"));
        assert!(trie.insert("Heron"));
        assert!(trie.insert("Stuff"));
        assert!(trie.insert("Alex"));
        print_trie(&trie);
        // println!("{trie:?}");
    }

    #[test]
    fn insert_2() {
        let mut trie = Trie::new();
        assert!(trie.insert("Hello"));
        assert!(trie.insert("World"));
        assert!(trie.insert("Hell"));
        assert!(!trie.insert("Hello"));
    }

    #[test]
    fn contains_1() {
        let mut trie = Trie::new();
        assert!(trie.insert("Hello"));
        assert!(trie.contains("Hello"));
    }

    #[test]
    fn contains_2() {
        let trie = Trie::new();
        assert!(!trie.contains("Hello"));
    }

    #[test]
    fn contains_3() {
        let mut trie = Trie::new();
        assert!(trie.insert("Hello"));
        assert!(!trie.contains("Hell"));
        assert!(!trie.contains("Help"));
        assert!(!trie.contains("E"));
        assert!(!trie.contains("LONG SEQUENCE OF CHARACTERS"));
        assert!(!trie.contains(""));
        assert!(trie.contains("Hello"));
    }

    #[test]
    fn contains_4() {
        let mut trie = Trie::new();
        assert!(trie.insert("Apple"));
        assert!(trie.insert("Adam"));
        assert!(trie.insert("Garden"));
        assert!(trie.insert("Gravel"));
        assert!(trie.insert("Ground"));
        assert!(trie.insert("Axel"));
        assert!(trie.insert("Alex"));
        assert!(trie.insert("Alexander"));

        assert!(trie.contains("Apple"));
        assert!(trie.contains("Adam"));
        assert!(trie.contains("Garden"));
        assert!(trie.contains("Gravel"));
        assert!(trie.contains("Ground"));
        assert!(trie.contains("Axel"));
        assert!(trie.contains("Alex"));
        assert!(trie.contains("Alexander"));

        assert!(!trie.contains("Ale"));
        assert!(!trie.contains("Gravity"));
        assert!(!trie.contains("Grav"));
        assert!(!trie.contains("Gro"));
        assert!(!trie.contains("G"));
        assert!(!trie.contains("Grounded"));
    }

    #[test]
    fn remove_1() {
        let mut trie = Trie::new();
        let had_value_to_remove = trie.remove("Nothing");
        assert!(!had_value_to_remove);
    }

    #[test]
    fn remove_2() {
        let mut trie = Trie::new();
        assert!(trie.insert("Hello"));
        assert!(trie.remove("Hello"));
    }
}
