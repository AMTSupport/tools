use std::path::PathBuf;
use std::sync::LazyLock;

pub static DRIVES: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    let mut drives = Vec::with_capacity(26);
    for x in 0..26 {
        let drive = format!("{}:", (x + 64) as u8 as char);
        let drive = PathBuf::from(drive);
        if drive.exists() {
            drives.push(drive);
        }
    }

    drives
});

pub static ROOT_DRIVE: LazyLock<&'static PathBuf> = LazyLock::new(|| DRIVES.first().unwrap());
