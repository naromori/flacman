use std::fs;
use std::path::{Path, PathBuf};

use crate::fserror::Result;
use crate::FsError;


/// Check if source file exists and is accessible
fn validate_source(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(FsError::PathNotFound(path.to_path_buf()));
    }

    if path.is_dir() {
        return Err(FsError::NotAFile(path.to_path_buf()));
    }

    // Check if readable
    fs::metadata(path).map_err(|e| FsError::Io(e))?;

    Ok(())
}

/// Check if destination is valid (parent exists, not same as source)
fn validate_destination(source: &Path, dest: &Path, allow_overwrite: bool) -> Result<()> {

    if let (Ok(src_canon), Ok(dst_canon)) = (fs::canonicalize(source), fs::canonicalize(dest)) {
        if src_canon == dst_canon {
            return Err(FsError::SameFile(dest.to_path_buf()));
        }
    }

    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            return Err(FsError::PathNotFound(parent.to_path_buf()));
        }
    }

    if dest.exists() && !allow_overwrite {
        return Err(FsError::FileAlreadyExists(dest.to_path_buf()));
    }

    Ok(())
}

fn validate_writable(path: &Path) -> Result<()> {
    if path.exists() {
        let metadata = fs::metadata(path)?;
        if metadata.permissions().readonly() {
            return Err(FsError::PermissionError(path.to_path_buf()));
        }
    }
    Ok(())
}

/// Copy file from source to destination
/// 
/// # Arguments
/// * `source` - Source file path
/// * `dest` - Destination file path
/// * `overwrite` - Whether to overwrite existing file
/// 
/// # Returns
/// The destination path on success
pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    dest: Q,
    overwrite: bool,
) -> Result<PathBuf> {
    let src = source.as_ref();
    let dst = dest.as_ref();

    validate_source(src)?;
    validate_destination(src, dst, overwrite)?;

    if overwrite && dst.exists() {
        validate_writable(dst)?;
    }

    fs::copy(src, dst)?;

    Ok(dst.to_path_buf())
}

/// Move file from source to destination
/// 
/// # Arguments
/// * `source` - Source file path
/// * `dest` - Destination file path
/// * `overwrite` - Whether to overwrite existing file
/// 
/// # Returns
/// The destination path on success
pub fn move_file<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    dest: Q,
    overwrite: bool,
) -> Result<PathBuf> {
    let src = source.as_ref();
    let dst = dest.as_ref();

    validate_source(src)?;
    validate_destination(src, dst, overwrite)?;

    if overwrite && dst.exists() {
        validate_writable(dst)?;
        fs::remove_file(dst)?;
    }

    match fs::rename(src, dst) {
        Ok(_) => Ok(dst.to_path_buf()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            fs::copy(src, dst)?;
            fs::remove_file(src)?;
            Ok(dst.to_path_buf())
        }
        Err(e) => Err(FsError::Io(e)),
    }
}

/// Create a symbolic link
/// 
/// # Arguments
/// * `source` - Source file path (target of the link)
/// * `dest` - Destination path (where the symlink will be created)
/// * `overwrite` - Whether to overwrite existing file
/// 
/// # Returns
/// The destination path on success
/// 
/// # Note
/// Unix-only
/// 
pub fn symlink_file<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    dest: Q,
    overwrite: bool,
) -> Result<PathBuf> {
    let src = source.as_ref();
    let dst = dest.as_ref();

    validate_source(src)?;

    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            return Err(FsError::PathNotFound(parent.to_path_buf()));
        }
    }

    if dst.exists() {
        if !overwrite {
            return Err(FsError::FileAlreadyExists(dst.to_path_buf()));
        }
        validate_writable(dst)?;
        fs::remove_file(dst)?;
    }

    std::os::unix::fs::symlink(src, dst)?;

    Ok(dst.to_path_buf())
}


/// Create a hard link
/// 
/// # Arguments
/// * `source` - Source file path
/// * `dest` - Destination path (where the hard link will be created)
/// * `overwrite` - Whether to overwrite existing file
/// 
/// # Returns
/// The destination path on success
/// 
/// # Note
/// Hard links must be on the same filesystem
pub fn hardlink_file<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    dest: Q,
    overwrite: bool,
) -> Result<PathBuf> {
    let src = source.as_ref();
    let dst = dest.as_ref();

    validate_source(src)?;

    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            return Err(FsError::PathNotFound(parent.to_path_buf()));
        }
    }

    if dst.exists() {
        if !overwrite {
            return Err(FsError::FileAlreadyExists(dst.to_path_buf()));
        }
        validate_writable(dst)?;
        fs::remove_file(dst)?;
    }

    fs::hard_link(src, dst)?;

    Ok(dst.to_path_buf())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferMode {
    /// Copy the file (leaves source intact)
    Copy,
    /// Move the file (removes source)
    Move,
    /// Create a symbolic link
    Symlink,
    /// Create a hard link
    Hardlink,
}

/// Generic transfer function that uses the specified mode
pub fn transfer_file<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    dest: Q,
    mode: TransferMode,
    overwrite: bool,
) -> Result<PathBuf> {
    match mode {
        TransferMode::Copy => copy_file(source, dest, overwrite),
        TransferMode::Move => move_file(source, dest, overwrite),
        TransferMode::Symlink => symlink_file(source, dest, overwrite),
        TransferMode::Hardlink => hardlink_file(source, dest, overwrite),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_copy_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");

        let mut file = File::create(&src).unwrap();
        file.write_all(b"test content").unwrap();

        let result = copy_file(&src, &dst, false).unwrap();
        assert_eq!(result, dst);
        assert!(src.exists());
        assert!(dst.exists());
    }

    #[test]
    fn test_copy_file_no_overwrite() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");

        File::create(&src).unwrap();
        File::create(&dst).unwrap();

        // Should fail without overwrite
        let result = copy_file(&src, &dst, false);
        assert!(matches!(result, Err(FsError::FileAlreadyExists(_))));
    }

    #[test]
    fn test_move_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");

        File::create(&src).unwrap();

        // Move
        let result = move_file(&src, &dst, false).unwrap();
        assert_eq!(result, dst);
        assert!(!src.exists());
        assert!(dst.exists());
    }

    #[test]
    fn test_hardlink_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("link.txt");

        File::create(&src).unwrap();

        // Create hardlink
        let result = hardlink_file(&src, &dst, false).unwrap();
        assert_eq!(result, dst);
        assert!(src.exists());
        assert!(dst.exists());
    }
}