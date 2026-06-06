use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub home: PathBuf,
    pub db_path: PathBuf,
}

pub fn resolve_paths() -> Result<AppPaths> {
    let override_home = env::var_os("TERM_PET_HOME").map(PathBuf::from);
    resolve_paths_from_home(override_home)
}

fn resolve_paths_from_home(override_home: Option<PathBuf>) -> Result<AppPaths> {
    let home = if let Some(path) = override_home {
        path
    } else {
        ProjectDirs::from("dev", "tty-pet", "tty-pet")
            .map(|dirs| dirs.data_local_dir().to_path_buf())
            .ok_or_else(|| anyhow!("could not resolve application data directory"))?
    };

    Ok(AppPaths {
        db_path: home.join("tty-pet.db"),
        home,
    })
}

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_pet_home_controls_database_path() {
        let paths = resolve_paths_from_home(Some(PathBuf::from("/tmp/tty-pet-test"))).unwrap();

        assert_eq!(paths.home, PathBuf::from("/tmp/tty-pet-test"));
        assert_eq!(paths.db_path, PathBuf::from("/tmp/tty-pet-test/tty-pet.db"));
    }
}
