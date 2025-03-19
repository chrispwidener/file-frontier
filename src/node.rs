use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
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
    pub size: u64,
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

        let mut node = Self {
            path,
            node_type,
            metadata,
            children: None,
            size: 0,
        };

        node.populate_children();
        node.calc_size();

        Ok(node)
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
    fn calc_size(&mut self) -> io::Result<()> {
        if self.is_file() {
            let metadata = fs::metadata(&self.path)?;
            self.size = metadata.size();
            Ok(())
        } else {
            let mut total = 0;
            // Populate children if not already done.
            if self.children.is_none() {
                self.populate_children()?;
            }
            if let Some(children) = &mut self.children {
                for child in children {
                    child.calc_size()?;
                    total += child.size;
                }
            }
            self.size = total;

            Ok(())
        }
    }
}

use std::fmt;

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.node_type {
            NodeType::File => {
                write!(
                    f,
                    "File: {} (size: {:?})",
                    self.path.display(),
                    self.size,
                )
            }
            NodeType::Directory => {
                write!(
                    f,
                    "Directory: {} (size: {:?})\n",
                    self.path.display(),
                    self.size
                )?;

                // If children are not populated, note that.
                if self.children.is_none() {
                    return write!(f, "  [Children not populated]");
                }

                // Separate children into file and directory vectors.
                let mut file_children = Vec::new();
                let mut dir_children = Vec::new();

                if let Some(children) = &self.children {
                    for child in children {
                        match child.node_type {
                            NodeType::File => file_children.push(child),
                            NodeType::Directory => dir_children.push(child),
                        }
                    }
                }

                // Display file children with path and size.
                if !file_children.is_empty() {
                    writeln!(f, "  File Children:")?;
                    for child in file_children {
                        writeln!(
                            f,
                            "    {} (size: {:?} bytes)",
                            child.path.display(),
                            child.size
                        )?;
                    }
                }

                // Display directory children with only path.
                if !dir_children.is_empty() {
                    writeln!(f, "  Directory Children:")?;
                    for child in dir_children {
                        writeln!(f, "    {}", child.path.display())?;
                    }
                }

                Ok(())
            }
        }
    }
}