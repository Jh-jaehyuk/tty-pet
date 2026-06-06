pub mod git;
pub mod identity;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ProjectIdentity {
    pub id: String,
    pub root_path: PathBuf,
    pub git_remote_url: Option<String>,
}

impl ProjectIdentity {
    pub fn display_name(&self) -> String {
        self.root_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| self.root_path.display().to_string())
    }
}
