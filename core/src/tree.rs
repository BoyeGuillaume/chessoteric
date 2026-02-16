use std::{num::NonZeroU32, ops::Deref};

#[derive(Debug, Clone)]
struct TreeNode<T> {
    value: T,
    next_siblings: Option<NonZeroU32>, // Root cannot be a sibling
    first_child: Option<NonZeroU32>,   // Leaf nodes cannot have children
    parent: u32,                       // Root has parent itself
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeNodeRef(u32);

impl TreeNodeRef {
    pub const ROOT: TreeNodeRef = TreeNodeRef(0);
}

#[derive(Debug, Clone)]
pub struct Tree<T> {
    container: Vec<TreeNode<T>>,
}

impl<T> Tree<T> {
    pub fn new(root_value: T) -> Self {
        Tree {
            container: vec![TreeNode {
                value: root_value,
                next_siblings: None,
                first_child: None,
                parent: 0,
            }],
        }
    }

    pub fn root(&self) -> TreeRef<'_, T> {
        TreeRef {
            tree: self,
            node_ref: TreeNodeRef(0),
        }
    }

    pub fn get(&self, node_ref: TreeNodeRef) -> TreeRef<'_, T> {
        TreeRef {
            tree: self,
            node_ref,
        }
    }

    pub fn get_mut(&mut self, node_ref: TreeNodeRef) -> TreeRefMut<'_, T> {
        TreeRefMut {
            tree: self,
            node_ref,
        }
    }
}

/// A reference to a node in the tree, which allows us to navigate the tree structure.
#[derive(Debug)]
pub struct TreeRef<'a, T> {
    tree: &'a Tree<T>,
    node_ref: TreeNodeRef,
}

impl<'a, T> Clone for TreeRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, T> Copy for TreeRef<'a, T> {}

impl<'a, T> Deref for TreeRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.tree.container[self.node_ref.0 as usize].value
    }
}

impl<'a, T> TreeRef<'a, T> {
    pub fn noderef(&self) -> TreeNodeRef {
        self.node_ref
    }

    pub fn child(&self) -> Option<TreeRef<'a, T>> {
        self.tree.container[self.node_ref.0 as usize]
            .first_child
            .map(|child_ref| TreeRef {
                tree: self.tree,
                node_ref: TreeNodeRef(child_ref.get()),
            })
    }

    pub fn next(&self) -> Option<TreeRef<'a, T>> {
        self.tree.container[self.node_ref.0 as usize]
            .next_siblings
            .map(|sibling_ref| TreeRef {
                tree: self.tree,
                node_ref: TreeNodeRef(sibling_ref.get()),
            })
    }

    pub fn parent(&self) -> Option<TreeRef<'a, T>> {
        let parent_index = self.tree.container[self.node_ref.0 as usize].parent;
        if self.node_ref.0 == 0 {
            None
        } else {
            Some(TreeRef {
                tree: self.tree,
                node_ref: TreeNodeRef(parent_index),
            })
        }
    }
}

/// A mutable reference to a node in the tree, which allows us to modify the tree structure.
#[derive(Debug)]
pub struct TreeRefMut<'a, T> {
    tree: &'a mut Tree<T>,
    node_ref: TreeNodeRef,
}

impl<'a, T> Deref for TreeRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.tree.container[self.node_ref.0 as usize].value
    }
}

impl<'a, T> std::ops::DerefMut for TreeRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree.container[self.node_ref.0 as usize].value
    }
}

impl<'a, T> TreeRefMut<'a, T> {
    pub fn noderef(&self) -> TreeNodeRef {
        self.node_ref
    }

    pub fn child(self) -> Option<TreeRefMut<'a, T>> {
        self.tree.container[self.node_ref.0 as usize]
            .first_child
            .map(|child_ref| TreeRefMut {
                tree: self.tree,
                node_ref: TreeNodeRef(child_ref.get()),
            })
    }

    pub fn child_noderef(&self) -> Option<TreeNodeRef> {
        self.tree.container[self.node_ref.0 as usize]
            .first_child
            .map(|child_ref| TreeNodeRef(child_ref.get()))
    }

    pub fn next(self) -> Option<TreeRefMut<'a, T>> {
        self.tree.container[self.node_ref.0 as usize]
            .next_siblings
            .map(|sibling_ref| TreeRefMut {
                tree: self.tree,
                node_ref: TreeNodeRef(sibling_ref.get()),
            })
    }

    pub fn next_noderef(&self) -> Option<TreeNodeRef> {
        self.tree.container[self.node_ref.0 as usize]
            .next_siblings
            .map(|sibling_ref| TreeNodeRef(sibling_ref.get()))
    }

    pub fn push_child(&mut self, value: T) -> TreeNodeRef {
        let new_node_index = self.tree.container.len() as u32;
        let current_first_child = self.tree.container[self.node_ref.0 as usize].first_child;

        self.tree.container.push(TreeNode {
            value,
            next_siblings: current_first_child,
            first_child: None,
            parent: self.node_ref.0,
        });

        let new_node_ref = TreeNodeRef(new_node_index);

        self.tree.container[self.node_ref.0 as usize].first_child =
            Some(NonZeroU32::new(new_node_index).unwrap());
        new_node_ref
    }
}
