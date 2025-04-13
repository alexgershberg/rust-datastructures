use crate::bplustree::BPlusTree;
use crate::bplustree::leaf::Leaf;
use crate::bplustree::node::Node;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ptr::NonNull;

pub fn create_leaf<K, V>(k: K, v: V) -> NonNull<Node<K, V>> {
    let leaf = Node::Leaf(Leaf {
        parent: None,
        data: vec![(k, v)],
    });
    unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(leaf))) }
}

pub unsafe fn cleanup_leaf<K, V>(ptr: NonNull<Node<K, V>>) {
    unsafe {
        let _ = Box::from_raw(ptr.as_ptr());
    }
}

pub fn print_bplustree<K, V>(tree: &BPlusTree<K, V>, options: DebugOptions)
where
    K: Ord + PartialOrd + Clone + Debug,
    V: Ord + PartialOrd + Clone + Debug,
{
    let Some(root) = tree.root else {
        println!("Empty");
        return;
    };

    let _ = tree
        .largest_key()
        .expect("If a tree is not empty, it's guaranteed to have at least a single value");

    unsafe { print_node(root, options) };
}

#[derive(Debug, Copy, Clone, Default)]
pub struct PtrDebugOptions {
    show_values: bool,
}

impl PtrDebugOptions {
    pub fn values(self) -> Self {
        Self { show_values: true }
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct ShowParent {
    internal: Option<PtrDebugOptions>,
    leaf: Option<PtrDebugOptions>,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DebugOptions {
    show_parent: ShowParent,
    override_padding: Option<usize>,
}

impl DebugOptions {
    pub fn internal_address(mut self) -> Self {
        let ptr_debug_options = if let Some(ptr_debug_options) = self.show_parent.internal {
            ptr_debug_options
        } else {
            PtrDebugOptions::default()
        };

        self.show_parent.internal = Some(ptr_debug_options);
        self
    }

    pub fn internal_values(mut self) -> Self {
        let ptr_debug_options = if let Some(ptr_debug_options) = self.show_parent.internal {
            ptr_debug_options.values()
        } else {
            PtrDebugOptions::default().values()
        };

        self.show_parent.internal = Some(ptr_debug_options);
        self
    }

    pub fn leaf_address(mut self) -> Self {
        let ptr_debug_options = if let Some(ptr_debug_options) = self.show_parent.leaf {
            ptr_debug_options
        } else {
            PtrDebugOptions::default()
        };

        self.show_parent.leaf = Some(ptr_debug_options);
        self
    }

    pub fn leaf_values(mut self) -> Self {
        let ptr_debug_options = if let Some(ptr_debug_options) = self.show_parent.leaf {
            ptr_debug_options.values()
        } else {
            PtrDebugOptions::default().values()
        };

        self.show_parent.leaf = Some(ptr_debug_options);
        self
    }

    pub fn all_address(self) -> Self {
        self.leaf_address().internal_address()
    }

    pub fn all_values(self) -> Self {
        self.internal_values().leaf_values()
    }

    pub fn override_padding(mut self, padding: usize) -> Self {
        self.override_padding = Some(padding);
        self
    }
}

pub unsafe fn print_node<K, V>(root: NonNull<Node<K, V>>, options: DebugOptions)
where
    K: Ord + PartialOrd + Clone + Debug,
    V: Ord + PartialOrd + Clone + Debug,
{
    let key_length = if let Some(padding) = options.override_padding {
        padding
    } else {
        4
    };
    let mut stack = VecDeque::from([(None, 0, false, root, -1)]);
    while let Some((pair, mut offset, ignore_offset, current_ptr, lvl)) = stack.pop_front() {
        let current = unsafe { current_ptr.as_ref() };
        if let Some((key, origin_ptr)) = pair {
            let line = if let Some(ptr_debug_options) = options.show_parent.internal {
                let formatted_ptr = unsafe { format_node_ptr(origin_ptr, ptr_debug_options) };
                format!("{} {key:key_length$?}  ->  ", formatted_ptr)
            } else {
                format!("{key:key_length$?}  ->  ")
            };

            offset += line.chars().count();
            let mut offset = offset;
            if ignore_offset {
                offset = 0;
            }
            print!("{:>offset$}", line);
        }

        let mut should_print_new_line = false;

        match current {
            Node::Internal(internal) => {
                for (index, (k, child_ptr)) in internal.links.iter().rev().enumerate() {
                    let last = index == internal.links.len() - 1;
                    let mut ignore_offset = false;
                    if last {
                        ignore_offset = true
                    }
                    stack.push_front((
                        Some((k, current_ptr)),
                        offset,
                        ignore_offset,
                        *child_ptr,
                        lvl + 1,
                    ));
                }
            }
            Node::Leaf(leaf) => {
                let mut first = true;
                for (k, v) in &leaf.data {
                    let line = if let Some(ptr_debug_options) = options.show_parent.leaf {
                        let formatted_ptr =
                            unsafe { format_node_ptr(current_ptr, ptr_debug_options) };
                        format!("{} {k:key_length$?}: {v:key_length$?}", formatted_ptr)
                    } else {
                        format!("{k:key_length$?}: {v:key_length$?}")
                    };

                    let mut offset = offset + line.chars().count();
                    if first {
                        offset = 0;
                        first = false;
                    }
                    println!("{line:>offset$}");
                }

                should_print_new_line = true;
            }
        }

        if should_print_new_line && !stack.is_empty() {
            println!()
        }
    }
}

/*
  [0x500000000] (123456, 10) -> [0x300000000] (123456, 10)
                                              (123456, 12)
                                              (123456, 13)
                                              (123456, 14)

  [0x900000000] (123456, 25) -> [0x700000000] (123456, 25)
                                              (123456, 35)
*/

pub unsafe fn format_node_ptr<K, V>(
    ptr: NonNull<Node<K, V>>,
    ptr_debug_options: PtrDebugOptions,
) -> String
where
    K: Ord + PartialOrd + Clone + Debug,
    V: Ord + PartialOrd + Clone + Debug,
{
    let n = unsafe { &*ptr.as_ptr() };
    let parent_ptr = n.parent_raw();
    if !ptr_debug_options.show_values {
        return format!("[{ptr:p} | parent: {parent_ptr:?}]");
    }

    if let Some(parent) = parent_ptr {
        let parent = unsafe { parent.as_ref() };
        let parent_data = match parent {
            Node::Internal(internal) => internal.links.iter().map(|(k, _)| k).collect::<Vec<_>>(),
            Node::Leaf(leaf) => leaf.data.iter().map(|(k, _)| k).collect::<Vec<_>>(),
        };
        format!("[{ptr:p} | parent: {:?}: {:?}]", parent_ptr, parent_data)
    } else {
        format!("[{ptr:p} | parent: {:?}]", parent_ptr)
    }
}

pub unsafe fn print_node_ptr<K, V>(ptr: NonNull<Node<K, V>>)
where
    K: Ord + PartialOrd + Clone + Debug,
    V: Ord + PartialOrd + Clone + Debug,
{
    unsafe {
        println!(
            "{}",
            format_node_ptr(ptr, PtrDebugOptions::default().values())
        );
    }
}

#[cfg(test)]
mod test {
    use crate::bplustree::debug::{
        DebugOptions, PtrDebugOptions, cleanup_leaf, create_leaf, format_node_ptr, print_node,
        print_node_ptr,
    };
    use crate::bplustree::internal::Internal;
    use crate::bplustree::node::Node;
    use std::ptr::NonNull;

    #[test]
    fn print_1() {
        let mut leaf1 = create_leaf(0, 0);
        let mut leaf2 = create_leaf(5, 1);
        let mut leaf3 = create_leaf(10, 2);

        let internal = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node::Internal(Internal {
                parent: Some(leaf1),
                links: vec![(0, leaf1), (5, leaf2), (10, leaf3)],
            }))))
        };

        unsafe {
            leaf1.as_mut().set_parent(Some(internal));
            leaf2.as_mut().set_parent(Some(internal));
            leaf3.as_mut().set_parent(Some(internal));

            print_node(internal, DebugOptions::default().all_values());
            println!();
            println!();
            println!();
            print_node(internal, DebugOptions::default().all_address());
            println!();
            println!();
            println!();
            print_node(internal, DebugOptions::default().leaf_address());
            println!();
            println!();
            println!();
            print_node(internal, DebugOptions::default().internal_address());
            println!();
            println!();
            println!();
            print_node(internal, DebugOptions::default().internal_values());

            cleanup_leaf(internal);
            cleanup_leaf(leaf1);
            cleanup_leaf(leaf2);
            cleanup_leaf(leaf3);
        }
    }
}
