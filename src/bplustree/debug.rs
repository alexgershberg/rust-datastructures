use crate::bplustree::BPlusTree;
use crate::bplustree::node::Node;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ptr::NonNull;

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
struct PtrDebugOptions {
    show_values: bool,
}

impl PtrDebugOptions {
    fn values(self) -> Self {
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
            PtrDebugOptions::default().values()
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

pub unsafe fn print_node<K, V>(ptr: NonNull<Node<K, V>>, options: DebugOptions)
where
    K: Ord + PartialOrd + Clone + Debug,
    V: Ord + PartialOrd + Clone + Debug,
{
    let key_length = if let Some(padding) = options.override_padding {
        padding
    } else {
        4
    };
    let mut stack = VecDeque::from([(None, 0, false, ptr, -1)]);
    while let Some((k, mut offset, ignore_offset, current_ptr, lvl)) = stack.pop_front() {
        let current = unsafe { current_ptr.as_ref() };
        if let Some(key) = k {
            let line = if let Some(ptr_debug_options) = options.show_parent.internal {
                let formatted_ptr = if let Some(parent_ptr) = current.parent_raw() {
                    unsafe { format_ptr(parent_ptr, ptr_debug_options) }
                } else {
                    "null".to_string()
                };
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
                for (index, (k, ptr)) in internal.links.iter().rev().enumerate() {
                    let last = index == internal.links.len() - 1;
                    let mut ignore_offset = false;
                    if last {
                        ignore_offset = true
                    }
                    stack.push_front((Some(k), offset, ignore_offset, *ptr, lvl + 1));
                }
            }
            Node::Leaf(leaf) => {
                let mut first = true;
                for (k, v) in &leaf.data {
                    let line = if let Some(ptr_debug_options) = options.show_parent.leaf {
                        let formatted_ptr = if let Some(parent) = leaf.parent_raw() {
                            unsafe { format_ptr(parent, ptr_debug_options) }
                        } else {
                            "null".to_string()
                        };
                        format!("({}) {k:key_length$?}: {v:key_length$?}", formatted_ptr)
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

pub unsafe fn format_ptr<K, V>(
    ptr: NonNull<Node<K, V>>,
    ptr_debug_options: PtrDebugOptions,
) -> String
where
    K: Ord + PartialOrd + Clone + Debug,
    V: Ord + PartialOrd + Clone + Debug,
{
    let n = unsafe { &*ptr.as_ptr() };
    if !ptr_debug_options.show_values {
        return format!("({ptr:p})",);
    }

    let data = match n {
        Node::Internal(internal) => internal.links.iter().map(|(k, _)| k).collect::<Vec<_>>(),
        Node::Leaf(leaf) => leaf.data.iter().map(|(k, _)| k).collect::<Vec<_>>(),
    };

    format!("({ptr:p}): {:?} | parent: {:?}", data, n.parent_raw())
}

pub unsafe fn print_ptr<K, V>(ptr: NonNull<Node<K, V>>)
where
    K: Ord + PartialOrd + Clone + Debug,
    V: Ord + PartialOrd + Clone + Debug,
{
    unsafe {
        println!("{}", format_ptr(ptr, PtrDebugOptions::default().values()));
    }
}
