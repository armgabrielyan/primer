use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn absolute(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return canonicalize(path);
    }

    let current_dir = std::env::current_dir().context("failed to read current directory")?;
    Ok(normalize(current_dir.join(path)))
}

pub fn canonicalize(path: &Path) -> Result<PathBuf> {
    let resolved =
        fs::canonicalize(path).with_context(|| format!("failed to resolve {}", path.display()))?;
    Ok(normalize(resolved))
}

pub fn normalize(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        normalize_windows(path)
    }

    #[cfg(not(windows))]
    {
        path
    }
}

#[cfg(windows)]
fn normalize_windows(path: PathBuf) -> PathBuf {
    let rendered = path.as_os_str().to_string_lossy();

    if let Some(stripped) = rendered.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{}", stripped));
    }

    if let Some(stripped) = rendered.strip_prefix(r"\\?\") {
        return PathBuf::from(stripped);
    }

    path
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    #[test]
    fn normalize_strips_verbatim_drive_prefix() {
        assert_eq!(
            super::normalize(std::path::PathBuf::from(r"\\?\C:\primer\workspace")),
            std::path::PathBuf::from(r"C:\primer\workspace")
        );
    }

    #[cfg(windows)]
    #[test]
    fn normalize_strips_verbatim_unc_prefix() {
        assert_eq!(
            super::normalize(std::path::PathBuf::from(r"\\?\UNC\server\share\primer")),
            std::path::PathBuf::from(r"\\server\share\primer")
        );
    }
}
