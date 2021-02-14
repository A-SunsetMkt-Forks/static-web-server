use std::error;
use std::path::{Path, PathBuf};

/// Validate and return a directory path.
pub fn get_valid_dirpath<P: AsRef<Path>>(path: P) -> Result<PathBuf, Box<dyn error::Error>>
where
    PathBuf: From<P>,
{
    match PathBuf::from(path) {
        v if !v.exists() => Result::Err(From::from(format!("path \"{:?}\" was not found", &v))),
        v if !v.is_dir() => {
            Result::Err(From::from(format!("path \"{:?}\" is not a directory", &v)))
        }
        v => Result::Ok(v),
    }
}

/// Get the directory name of a valid directory path.
pub fn get_dirname<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn error::Error>>
where
    PathBuf: From<P>,
{
    let path = match get_valid_dirpath(path) {
        Err(e) => return Result::Err(e),
        Ok(v) => v,
    };

    match path.iter().last() {
        Some(v) => Result::Ok(v.to_str().unwrap().to_string()),
        _ => Result::Err(From::from(format!(
            "directory name for path \"{:?}\" was not determined",
            path,
        ))),
    }
}

#[cfg(not(windows))]
pub fn adjust_canonicalization<P: AsRef<std::path::Path>>(p: P) -> String {
    p.as_ref().display().to_string()
}

#[cfg(windows)]
pub fn adjust_canonicalization<P: AsRef<std::path::Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.as_ref().display().to_string();
    if p.starts_with(VERBATIM_PREFIX) {
        p[VERBATIM_PREFIX.len()..].to_string()
    } else {
        p
    }
}
