use std::fs::File;

struct FileInfo {
    base: File,
    notify_only: bool
}

impl FileInfo {
    fn new(base: File, notify_only: bool) -> Self {
        Self {
            base,
            notify_only
        }
    }
}

