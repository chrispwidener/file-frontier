use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Represents whether a node is a file or a directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
}

/// A struct to hold extended metadata about a file or directory.
#[derive(Debug, Clone)]
pub struct ExtendedMetadata {
    /// For files, this is the file size. For directories, this can be the cumulative size.
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub created: Option<SystemTime>,
    // Additional metadata such as permissions could be added here.
}

impl ExtendedMetadata {
    /// Create extended metadata for the given path.
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        Ok(Self {
            size: metadata.len(),
            modified: metadata.modified().ok(),
            accessed: metadata.accessed().ok(),
            created: metadata.created().ok(),
        })
    }
}

/// A Node in the directory tree.
#[derive(Debug, Clone)]
pub struct Node {
    /// Filesystem path of the node.
    pub path: PathBuf,
    /// Whether the node is a file or a directory.
    pub node_type: NodeType,
    /// Extended metadata for searching and reporting.
    pub metadata: ExtendedMetadata,
    /// Child nodes, if any. Directories can have children.
    pub children: Option<Vec<Node>>,
    /// Self Size if File, Cumulative size of all children if Directory.
    pub size: Option<u32>,
}

impl Node {
    /// Create a new Node from a given path.
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let metadata = ExtendedMetadata::from_path(&path)?;
        let node_type = if fs::metadata(&path)?.is_dir() {
            NodeType::Directory
        } else {
            NodeType::File
        };
        Ok(Self {
            path,
            node_type,
            metadata,
            children: None,
        })
    }

    /// Returns `true` if this node is a file.
    pub fn is_file(&self) -> bool {
        matches!(self.node_type, NodeType::File)
    }

    /// Returns `true` if this node is a directory.
    pub fn is_dir(&self) -> bool {
        matches!(self.node_type, NodeType::Directory)
    }

    /// Populate the node’s children from the file system.
    /// For a directory, reads its contents and creates child nodes.
    pub fn populate_children(&mut self) -> io::Result<()> {
        if self.is_dir() {
            let mut childs = Vec::new();
            for entry in fs::read_dir(&self.path)? {
                let entry = entry?;
                let child_path = entry.path();
                let child_node = Node::new(child_path)?;
                childs.push(child_node);
            }
            self.children = Some(childs);
        }
        Ok(())
    }

    /// Recursively updates the size of this node.
    /// For directories, the size is the sum of sizes of all children.
    pub fn update_size(&mut self) -> io::Result<u64> {
        if self.is_file() {
            self.metadata = ExtendedMetadata::from_path(&self.path)?;
            Ok(self.metadata.size)
        } else {
            let mut total = 0;
            // Populate children if not already done.
            if self.children.is_none() {
                self.populate_children()?;
            }
            if let Some(children) = &mut self.children {
                for child in children {
                    total += child.update_size()?;
                }
            }
            self.metadata.size = total;
            Ok(total)
        }
    }
}