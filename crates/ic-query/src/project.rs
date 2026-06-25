use std::path::{Path, PathBuf};

pub fn icp_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(path) = std::env::var("ICQ_ICP_ROOT") {
        return Ok(PathBuf::from(path).canonicalize()?);
    }

    let current_dir = std::env::current_dir()?.canonicalize()?;
    if let Some(root) = discover_icp_root_from(&current_dir) {
        return Ok(root);
    }
    Ok(current_dir)
}

fn discover_icp_root_from(start: &Path) -> Option<PathBuf> {
    let mut cursor = Some(start);
    while let Some(path) = cursor {
        if path.join("icp.yaml").is_file() || path.join("dfx.json").is_file() {
            return Some(path.to_path_buf());
        }
        cursor = path.parent();
    }
    None
}
