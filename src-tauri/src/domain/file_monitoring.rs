use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

pub const EVENT_DEBOUNCE_MS: u64 = 500;

pub fn validate_watch_path(path: &Path) -> Result<PathBuf, String> {
    let normalized = path
        .canonicalize()
        .map_err(|_| "경로를 찾을 수 없거나 접근할 수 없습니다.".to_string())?;
    if !normalized.is_dir() {
        return Err("폴더만 감시할 수 있습니다.".to_string());
    }
    if normalized.parent().is_none()
        || normalized
            == std::env::var_os("HOME")
                .map(PathBuf::from)
                .and_then(|home| home.canonicalize().ok())
                .unwrap_or_default()
    {
        return Err("시스템 루트 또는 사용자 홈 전체는 감시할 수 없습니다.".to_string());
    }
    if normalized.is_symlink() {
        return Err("심볼릭 링크는 감시할 수 없습니다.".to_string());
    }
    Ok(normalized)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileEventKind {
    Created,
    Modified,
    Deleted,
    Renamed,
    MetadataChanged,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedFileEvent {
    pub monitored_path: PathBuf,
    pub path: PathBuf,
    pub kind: FileEventKind,
    pub observed_at_ms: u128,
}

pub fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub fn is_duplicate(previous: &NormalizedFileEvent, next: &NormalizedFileEvent) -> bool {
    previous.monitored_path == next.monitored_path
        && previous.path == next.path
        && previous.kind == next.kind
        && next.observed_at_ms.saturating_sub(previous.observed_at_ms) <= EVENT_DEBOUNCE_MS as u128
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hashes_streamed_file() {
        let file = std::env::temp_dir().join("nightingale-hash-test.txt");
        std::fs::write(&file, b"nightingale").expect("fixture");
        assert_eq!(sha256_file(&file).expect("hash").len(), 64);
        std::fs::remove_file(file).expect("cleanup");
    }
}
