use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

pub struct Symlink;

impl Symlink {
    /// Checks whether a path exists, including broken symbolic links.
    ///
    /// # Errors
    ///
    /// Returns an error if the path metadata cannot be read for a reason other
    /// than the path not existing, such as insufficient permissions or an I/O
    /// failure.
    pub fn exists(path: &Path) -> Result<bool, String> {
        match fs::symlink_metadata(path) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
            Err(e) => Err(format!(
                "Could not check existence of {}, {e}",
                path.display()
            )),
        }
    }

    /// Creates a symbolic link.
    ///
    /// If `destination` already exists, it is removed first.
    ///
    /// # Errors
    ///
    /// Returns an error if checking the destination fails, removing an existing
    /// destination fails, or creating the symbolic link fails.
    pub fn create(source: &Path, destination: &Path) -> Result<(), String> {
        if Self::exists(destination)? {
            fs::remove_file(destination)
                .map_err(|e| format!("Failed to remove {}: {e}", destination.display()))?;
        }

        Command::new("ln")
            .arg("-s")
            .arg(source)
            .arg(destination)
            .output()
            .map_err(|e| {
                format!(
                    "Failed to create symlink from {} to {}: {e}",
                    source.display(),
                    destination.display()
                )
            })?;

        Ok(())
    }
}
