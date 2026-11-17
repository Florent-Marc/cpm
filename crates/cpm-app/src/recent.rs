use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct RecentPacks {
    pub paths: Vec<PathBuf>,
    storage: Option<PathBuf>,
}
impl RecentPacks {
    pub fn load(storage: PathBuf) -> (Self, Option<String>) {
        let mut recent = Self {
            paths: Vec::new(),
            storage: Some(storage.clone()),
        };
        let paths: Vec<PathBuf> = match std::fs::read(&storage) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(paths) => paths,
                Err(error) => return (recent, Some(format!("Historique des packs : {error}"))),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (recent, None),
            Err(error) => return (recent, Some(format!("Historique des packs : {error}"))),
        };
        for path in paths {
            let normalized = std::fs::canonicalize(&path).unwrap_or(path);
            if !recent.paths.iter().any(|p| same_path(p, &normalized)) {
                recent.paths.push(normalized);
            }
            if recent.paths.len() == 5 {
                break;
            }
        }
        (recent, None)
    }
    pub fn remember(&mut self, path: &Path) -> Result<(), String> {
        let path =
            std::fs::canonicalize(path).map_err(|e| format!("Historique des packs : {e}"))?;
        self.paths.retain(|p| !same_path(p, &path));
        self.paths.insert(0, path);
        self.paths.truncate(5);
        if let Some(storage) = &self.storage {
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                let mut file = tempfile::NamedTempFile::new_in(
                    storage.parent().ok_or("Dossier introuvable")?,
                )?;
                serde_json::to_writer_pretty(file.as_file_mut(), &self.paths)?;
                file.as_file().sync_all()?;
                file.persist(storage)?;
                Ok(())
            })();
            result.map_err(|e| format!("Enregistrement de l’historique des packs : {e}"))?;
        }
        Ok(())
    }
}
fn same_path(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        left.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_five_distinct_packs_newest_first_after_restart() {
        let temp = tempfile::tempdir().unwrap();
        let storage = temp.path().join("recent-packs.json");
        let (mut recent, warning) = RecentPacks::load(storage.clone());
        assert!(warning.is_none());
        for n in 0..7 {
            let path = temp.path().join(format!("pack{n}"));
            std::fs::create_dir(&path).unwrap();
            recent.remember(&path).unwrap();
        }
        recent.remember(&temp.path().join("pack4/.")).unwrap();
        let (loaded, warning) = RecentPacks::load(storage);
        assert!(warning.is_none());
        let names: Vec<_> = loaded
            .paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["pack4", "pack6", "pack5", "pack3", "pack2"]);
        assert!(recent.remember(&temp.path().join("missing.zip")).is_err());
        assert_eq!(recent.paths, loaded.paths);
    }
    #[test]
    fn malformed_history_is_reported_without_preventing_startup() {
        let temp = tempfile::tempdir().unwrap();
        let storage = temp.path().join("recent-packs.json");
        std::fs::write(&storage, "broken").unwrap();
        let (recent, warning) = RecentPacks::load(storage.clone());
        assert!(warning.is_some());
        assert!(recent.paths.is_empty());
        assert_eq!(std::fs::read_to_string(storage).unwrap(), "broken");
    }
}
