use std::io;
use std::path::{Path, PathBuf};

use crate::node::Node;

/// An in-memory representation of a directory tree.
pub struct Tree {
    /// The root node of the tree.
    pub head: Node,
    // In lieu of a mutable “focus” pointer, we provide iterator and search methods.
}

impl Tree {
    /// Create a new tree from a given root path.
    pub fn new(root: &Path) -> io::Result<Self> {
        let head = Node::new(root.to_path_buf())?;
        Ok(Self { head })
    }

    /// Returns an iterator over all nodes in the tree using depth-first search.
    pub fn iter(&self) -> TreeIterator {
        TreeIterator {
            stack: vec![&self.head],
        }
    }

    /// Refreshes the tree structure by re-populating children and updating sizes.
    pub fn refresh(&mut self) -> io::Result<()> {
        self.head.populate_children()?;
        self.head.update_size()?;
        Ok(())
    }

    /// Search for nodes matching a given predicate.
    pub fn search<F>(&self, predicate: F) -> Vec<&Node>
    where
        F: Fn(&Node) -> bool,
    {
        self.iter().filter(|node| predicate(node)).collect()
    }

    /// Create a new directory at the given relative path from the tree’s root.
    /// This will update the on-disk structure and refresh the in-memory tree.
    pub fn create_dir(&mut self, rel_path: &Path) -> io::Result<Node> {
        let new_path = self.head.path.join(rel_path);
        std::fs::create_dir_all(&new_path)?;
        // Refresh the tree to include the new directory.
        self.refresh()?;
        // Retrieve and return the new node.
        self.get_node(&new_path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Node not found"))
    }

    /// Retrieve a node by its path, if it exists in the tree.
    pub fn get_node(&self, path: &Path) -> Option<&Node> {
        self.iter().find(|node| node.path == path)
    }
}

/// An iterator that traverses the tree in a depth-first manner.
pub struct TreeIterator<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> Iterator for TreeIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop()?;
        if let Some(children) = &current.children {
            for child in children {
                self.stack.push(child);
            }
        }
        Some(current)
    }
}