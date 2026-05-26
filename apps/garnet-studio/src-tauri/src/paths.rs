use std::path::{Path, PathBuf};

pub fn find_garnet_cli() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("GARNET_CLI") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(executable_name("garnet"));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    if let Some(root) = find_repo_root() {
        for profile in ["release", "debug"] {
            let candidate = root
                .join("target")
                .join(profile)
                .join(executable_name("garnet"));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    find_on_path(executable_name("garnet"))
}

pub fn find_repo_root() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("GARNET_REPO") {
        let candidate = PathBuf::from(path);
        if is_repo_root(&candidate) {
            return Some(candidate);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(root) = walk_up_for_workspace(exe) {
            return Some(root);
        }
    }

    std::env::current_dir().ok().and_then(walk_up_for_workspace)
}

pub fn evidence_base_dir() -> PathBuf {
    let desktop = dirs::desktop_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Desktop")
    });
    desktop.join("dogfood").join("garnet-studio-windows-linux")
}

pub fn domain_matrix_evidence_base_dir() -> PathBuf {
    if let Ok(path) = std::env::var("GARNET_STUDIO_DOMAIN_MATRIX_ROOT") {
        return PathBuf::from(path);
    }
    default_domain_matrix_evidence_base_dir()
}

fn default_domain_matrix_evidence_base_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Desktop")
        .join("dogfood")
        .join("garnet-studio-domain-matrix")
}

pub fn python_cmd() -> &'static str {
    if cfg!(windows) {
        "python"
    } else {
        "python3"
    }
}

fn executable_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

fn find_on_path(name: String) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(&name))
        .find(|candidate| candidate.exists())
}

fn walk_up_for_workspace(start: PathBuf) -> Option<PathBuf> {
    let mut dir = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start
    };

    for _ in 0..12 {
        if is_repo_root(&dir) {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn is_repo_root(path: &Path) -> bool {
    let cargo = path.join("Cargo.toml");
    let scripts = path
        .join("scripts")
        .join("garnet_windows_linux_studio_status.py");
    if !cargo.exists() || !scripts.exists() {
        return false;
    }
    std::fs::read_to_string(cargo)
        .map(|contents| contents.contains("[workspace]"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_path_uses_windows_linux_contract_root() {
        let path = evidence_base_dir().to_string_lossy().replace('\\', "/");
        assert!(path.ends_with("Desktop/dogfood/garnet-studio-windows-linux"));
    }

    #[test]
    fn domain_matrix_evidence_path_uses_readiness_scanned_root() {
        let path = domain_matrix_evidence_base_dir()
            .to_string_lossy()
            .replace('\\', "/");
        assert!(path.ends_with("Desktop/dogfood/garnet-studio-domain-matrix"));
    }

    #[test]
    fn default_domain_matrix_evidence_path_matches_python_scanner_default() {
        let expected = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Desktop")
            .join("dogfood")
            .join("garnet-studio-domain-matrix");
        assert_eq!(default_domain_matrix_evidence_base_dir(), expected);
    }
}
