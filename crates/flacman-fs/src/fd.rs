use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{fserror::Result, FsError};


/// Walk directory and return iterator over files in that directory
/// 
/// # Arguments
/// * `path` - Path to walk from
/// 
/// # Returns
/// Iterator over files in `path` directory
/// 
/// # Errors
/// * `FsError::PathNotFound` - Path doesn't exist
/// * `FsError::NotADirectory` - Path is a file, not a directory
/// * Iterator items may contain `FsError::WalkDir` for errors during traversal
pub fn walkdir<P: AsRef<Path>>(
    path: P,
) -> Result<impl Iterator<Item = Result<PathBuf>>> {
    let walk_path: &Path = path.as_ref();

    // Validate path upfront
    if !walk_path.exists() {
        return Err(FsError::PathNotFound(walk_path.to_path_buf()));
    }

    if walk_path.is_file() {
        return Err(FsError::NotADirectory(walk_path.to_path_buf()));
    }

    // Create iterator that propagates errors instead of dropping them
    let iter = WalkDir::new(walk_path)
        .into_iter()
        .filter_map(|entry_result| {
            match entry_result {
                Ok(entry) => {
                    // Only include files, skip directories
                    if entry.file_type().is_file() {
                        Some(Ok(entry.path().to_path_buf()))
                    } else {
                        None
                    }
                }

                Err(e) => Some(Err(FsError::WalkDir(e))),
            }
        });

    Ok(iter)
}

/// Walk directory but silently skip errors (useful for user-facing operations)
/// 
/// Use this when you want to be permissive about filesystem errors
/// (e.g., permission denied on some subdirectories)
pub fn walkdir_lenient<P: AsRef<Path>>(
    path: P,
) -> Result<impl Iterator<Item = PathBuf>> {
    let walk_path: &Path = path.as_ref();

    if !walk_path.exists() {
        return Err(FsError::PathNotFound(walk_path.to_path_buf()));
    }

    if walk_path.is_file() {
        return Err(FsError::NotADirectory(walk_path.to_path_buf()));
    }

    let iter = WalkDir::new(walk_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf());

    Ok(iter)
}

pub fn find_match_one<P: AsRef<Path>>(
    search_path: P,
    target_file: &Path,
) -> Result<Option<PathBuf>> {
    for result in walkdir(search_path)? {
        let file = result?;

        if file.file_name() == target_file.file_name() 
            && file.extension() == target_file.extension() 
        {
            return Ok(Some(file));
        }
    }

    Ok(None)
}

pub fn find_match_all<P: AsRef<Path>>(
    search_path: P,
    target_file: &Path,
) -> Result<Vec<PathBuf>> {
    let mut matches = Vec::new();

    for result in walkdir(search_path)? {
        let path = result?;

        if path.file_name() == target_file.file_name() 
            && path.extension() == target_file.extension() 
        {
            matches.push(path);
        }
    }

    Ok(matches)
}

pub fn find_ext<P: AsRef<Path>>(
    search_path: P,
    target_extension: &str,
) -> Result<Vec<PathBuf>> {
    let mut matches = Vec::new();

    for result in walkdir(search_path)? {
        let path = result?;

        if let Some(ext) = path.extension() {
            if ext.eq_ignore_ascii_case(target_extension) {
                matches.push(path);
            }
        }
    }

    Ok(matches)
}

pub fn find_pattern<P: AsRef<Path>>(
    search_path: P,
    pattern: &str,
) -> Result<Vec<PathBuf>> {
    let mut matches = Vec::new();

    for result in walkdir(search_path)? {
        let path = result?;

        if let Some(path_str) = path.to_str() {
            if path_str.contains(pattern) {
                matches.push(path);
            }
        }
    }

    Ok(matches)
}

/// Find all audio files in a directory
/// 
/// Searches for common audio file extensions (flac, mp3, m4a, ogg, opus, wav, aac, wma)
pub fn find_audio_files<P: AsRef<Path>>(search_path: P) -> Result<Vec<PathBuf>> {
    const AUDIO_EXTS: &[&str] = &["flac", "mp3", "m4a", "ogg", "opus", "wav", "aac", "wma"];
    
    let mut matches = Vec::new();

    for result in walkdir_lenient(search_path)? {
        let path = result;

        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if AUDIO_EXTS.contains(&ext_str.to_lowercase().as_str()) {
                    matches.push(path);
                }
            }
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File};
    use tempfile::tempdir;

    #[test]
    fn test_walkdir_nonexistent() {
        let result = walkdir("/nonexistent/path/xyz");
        assert!(matches!(result, Err(FsError::PathNotFound(_))));
    }

    #[test]
    fn test_walkdir_file_not_directory() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("file.txt");
        File::create(&file).unwrap();

        let result = walkdir(&file);
        assert!(matches!(result, Err(FsError::NotADirectory(_))));
    }

    #[test]
    fn test_find_audio_files() {
        let dir = tempdir().unwrap();
        
        // Create test files
        File::create(dir.path().join("song.flac")).unwrap();
        File::create(dir.path().join("track.mp3")).unwrap();
        File::create(dir.path().join("readme.txt")).unwrap();

        let result = find_audio_files(dir.path()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_find_ext() {
        let dir = tempdir().unwrap();
        
        File::create(dir.path().join("one.flac")).unwrap();
        File::create(dir.path().join("two.FLAC")).unwrap();
        File::create(dir.path().join("three.mp3")).unwrap();

        let result = find_ext(dir.path(), "flac").unwrap();
        assert_eq!(result.len(), 2); // Case-insensitive
    }
}