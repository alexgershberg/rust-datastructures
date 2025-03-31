use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::{Debug, Formatter};
use std::ptr::NonNull;

pub struct Trie {
    root: Option<NonNull<Node>>,
}

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
        todo!()
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

impl Debug for Trie {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Trie").field("root", &self.root).finish()
    }
}

fn print_trie(trie: &Trie) {
    println!("root: {:?}", trie.root);

    if let Some(root) = trie.root {
        let node = unsafe { root.as_ref() };
        println!("children: {:#?}", node.children);
        println!("terminal: {}", node.terminal)
    }
}

#[cfg(test)]
mod tests {
    use crate::trie::{Trie, print_trie};

    #[test]
    fn debug_1() {
        let mut trie = Trie::new();
        trie.insert("Alex");
        println!("{trie:#?}");
    }

    #[test]
    fn insert_1() {
        let mut trie = Trie::new();
        trie.insert("Hello");
        trie.insert("Stop");
        trie.insert("Help");
        trie.insert("Heron");
        trie.insert("Stuff");
        trie.insert("Alex");
        print_trie(&trie);
    }
}
