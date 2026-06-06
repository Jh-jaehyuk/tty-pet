use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::project::{git, ProjectIdentity};

pub fn resolve_current_project() -> Result<ProjectIdentity> {
    resolve_project(env::current_dir()?)
}

pub fn resolve_project(cwd: PathBuf) -> Result<ProjectIdentity> {
    let git_root = git::root_for(&cwd);
    let root_path = match git_root {
        Some(path) => normalize_path(PathBuf::from(path)),
        None => normalize_path(cwd),
    };
    let git_remote_url = git::remote_url(&root_path);
    let id = stable_project_id(&root_path);

    Ok(ProjectIdentity {
        id,
        root_path,
        git_remote_url,
    })
}

fn normalize_path(path: PathBuf) -> PathBuf {
    std::fs::canonicalize(&path).unwrap_or(path)
}

pub fn stable_project_id(path: &Path) -> String {
    let input = path.to_string_lossy();
    let mut hash = 0xcbf29ce484222325_u64;

    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_project_id_is_repeatable() {
        let path = PathBuf::from("/tmp/example-project");

        assert_eq!(stable_project_id(&path), stable_project_id(&path));
    }

    #[test]
    fn stable_project_id_changes_with_path() {
        let first = stable_project_id(Path::new("/tmp/one"));
        let second = stable_project_id(Path::new("/tmp/two"));

        assert_ne!(first, second);
    }
}
