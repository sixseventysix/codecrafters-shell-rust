use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Find an executable in PATH
pub fn find_in_path(command: &str) -> Option<String> {
    let path_env = env::var("PATH").ok()?;

    for dir in path_env.split(':') {
        let full_path = Path::new(dir).join(command);

        if let Ok(metadata) = full_path.metadata() {
            if metadata.is_file() {
                let permissions = metadata.permissions();
                if permissions.mode() & 0o111 != 0 {
                    return Some(full_path.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}
