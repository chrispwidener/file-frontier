use notify::{Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::io;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

use crate::tree::Tree;

/// A simple filesystem watcher that monitors a path and refreshes the Tree on changes.
///
/// In this basic example the watcher runs in a blocking loop; for production use
/// you might run it on its own thread and communicate with the Tree via a shared state
/// (e.g., using Arc<Mutex<Tree>>).
pub struct FsWatcher {
    watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
}

impl FsWatcher {
    /// Create a new filesystem watcher.
    pub fn new() -> NotifyResult<Self> {
        let (tx, rx) = channel();
        // Using a delay of 2 seconds between events.
        let watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;
        Ok(Self { watcher, rx })
    }

    /// Start watching the specified path and refresh the tree on any filesystem event.
    ///
    /// This function is blocking. In practice, you might want to spawn a thread for this.
    pub fn watch(&mut self, path: &PathBuf, tree: &mut Tree) -> io::Result<()> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        println!("Started watching {:?}", path);

        loop {
            match self.rx.recv() {
                Ok(Ok(event)) => {
                    println!("Filesystem event: {:?}", event);
                    // For simplicity, we refresh the entire tree on any event.
                    tree.refresh()?;
                }
                Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
                Err(e) => {
                    eprintln!("Watch channel error: {:?}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}