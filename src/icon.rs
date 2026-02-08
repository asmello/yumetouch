use std::fs;
use std::path::PathBuf;

const ICON_BYTES: &[u8] = include_bytes!("../resources/icon.png");

pub fn ensure_icon() -> PathBuf {
    let cache_dir = PathBuf::from(std::env::var("HOME").expect("HOME not set"))
        .join("Library/Caches/yumetouch");
    let icon_path = cache_dir.join("icon.png");

    if icon_path
        .metadata()
        .is_ok_and(|m| m.len() == ICON_BYTES.len() as u64)
    {
        return icon_path;
    }

    if let Err(e) = fs::create_dir_all(&cache_dir) {
        log::warn!("failed to create icon cache dir: {e}");
        return icon_path;
    }

    if let Err(e) = fs::write(&icon_path, ICON_BYTES) {
        log::warn!("failed to write icon cache: {e}");
    }

    icon_path
}
