//! Vérification des releases publiques du dépôt. Les anciennes releases Java
//! et les préversions ne sont jamais proposées comme mises à jour Rust.
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

const API: &str = "https://api.github.com/repos/Florent-Marc/cpm/releases?per_page=100";
const DOWNLOAD_PREFIX: &str = "https://github.com/Florent-Marc/cpm/releases/download/";
const PAGE_PREFIX: &str = "https://github.com/Florent-Marc/cpm/releases/tag/";
const MAX_DOWNLOAD: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    digest: Option<String>,
    size: u64,
}
#[derive(Debug, Deserialize)]
pub struct Release {
    tag_name: String,
    html_url: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}
#[derive(Debug, Clone)]
pub struct AvailableUpdate {
    pub version: Version,
    pub page_url: String,
    asset: Option<Asset>,
}
impl AvailableUpdate {
    pub fn can_install(&self) -> bool {
        cfg!(all(target_os = "windows", target_arch = "x86_64"))
            && self.asset.as_ref().is_some_and(valid_asset)
    }
}

pub struct PreparedUpdate {
    directory: tempfile::TempDir,
    executable: PathBuf,
    target: PathBuf,
}

#[derive(Default)]
pub struct Updater {
    check: Option<Receiver<Result<Option<AvailableUpdate>, String>>>,
    download: Option<Receiver<Result<PreparedUpdate, String>>>,
    pub available: Option<AvailableUpdate>,
    pub message: String,
    next_check: Option<std::time::Instant>,
}
impl Updater {
    pub fn check(&mut self) {
        if self.check.is_some() || self.busy() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        self.check = Some(receiver);
        self.next_check = Some(std::time::Instant::now() + Duration::from_secs(3600));
        self.message = "Vérification des mises à jour…".into();
        std::thread::spawn(move || {
            let _ = sender.send(fetch_update());
        });
    }
    pub fn checking(&self) -> bool {
        self.check.is_some()
    }
    pub fn busy(&self) -> bool {
        self.download.is_some()
    }
    pub fn start_download(&mut self) {
        if self.busy() {
            return;
        }
        let Some(release) = self.available.clone() else {
            return;
        };
        if !release.can_install() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        self.download = Some(receiver);
        self.message = "Téléchargement de la mise à jour…".into();
        std::thread::spawn(move || {
            let _ = sender.send(download_update(&release));
        });
    }
    pub fn poll(&mut self) -> Option<Result<PreparedUpdate, String>> {
        if self
            .next_check
            .is_some_and(|next| std::time::Instant::now() >= next)
        {
            self.check();
        }
        if let Some(receiver) = &self.check {
            let result = match receiver.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => Some(Err("Vérification interrompue".into())),
            };
            if let Some(result) = result {
                self.check = None;
                match result {
                    Ok(available) => {
                        self.message = if available.is_some() {
                            "Nouvelle version disponible"
                        } else {
                            "Aucune version Rust plus récente publiée"
                        }
                        .into();
                        self.available = available;
                    }
                    Err(error) => self.message = format!("Mises à jour indisponibles : {error}"),
                }
            }
        }
        let result = match self.download.as_ref()?.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Err("Téléchargement interrompu".into()),
        };
        self.download = None;
        Some(result)
    }
}

fn agent(timeout: u64) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(timeout)))
        .build()
        .into()
}
fn fetch_update() -> Result<Option<AvailableUpdate>, String> {
    let releases: Vec<Release> = agent(10)
        .get(API)
        .header("User-Agent", concat!("CPM/", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_json()
        .map_err(|e| e.to_string())?;
    let current = Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
    Ok(select_update(releases, &current))
}

fn platform_asset() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "cpm-windows-x86_64.exe",
        ("linux", "x86_64") => "cpm-linux-x86_64",
        ("macos", "aarch64") => "cpm-macos-aarch64",
        _ => "unsupported-platform",
    }
}
fn valid_asset(asset: &Asset) -> bool {
    let digest_ok = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()));
    asset.browser_download_url.starts_with(DOWNLOAD_PREFIX)
        && digest_ok
        && asset.size > 0
        && asset.size <= MAX_DOWNLOAD
}
fn select_update(releases: Vec<Release>, current: &Version) -> Option<AvailableUpdate> {
    releases
        .into_iter()
        .filter_map(|release| {
            if release.draft || release.prerelease || !release.html_url.starts_with(PAGE_PREFIX) {
                return None;
            }
            let tag = release
                .tag_name
                .strip_prefix("rust-v")
                .or_else(|| release.tag_name.strip_prefix("rust-"));
            let is_rust = tag.is_some()
                || release
                    .assets
                    .iter()
                    .any(|a| a.name == "cpm-windows-x86_64.exe");
            if !is_rust {
                return None;
            }
            let version =
                Version::parse(tag.unwrap_or(release.tag_name.trim_start_matches('v'))).ok()?;
            if !version.pre.is_empty() || &version <= current {
                return None;
            }
            let asset = release
                .assets
                .into_iter()
                .find(|a| a.name == platform_asset());
            Some(AvailableUpdate {
                version,
                page_url: release.html_url,
                asset,
            })
        })
        .max_by(|a, b| a.version.cmp(&b.version))
}

fn verify_digest(bytes: &[u8], expected: &str) -> bool {
    format!("{:x}", Sha256::digest(bytes)).eq_ignore_ascii_case(expected)
}
fn download_update(release: &AvailableUpdate) -> Result<PreparedUpdate, String> {
    let asset = release
        .asset
        .as_ref()
        .filter(|asset| valid_asset(asset))
        .ok_or("Exécutable compatible vérifiable absent de la release")?;
    let target = std::env::current_exe().map_err(|e| e.to_string())?;
    let parent = target.parent().ok_or("Dossier du programme introuvable")?;
    let directory = tempfile::Builder::new()
        .prefix(".cpm-update-")
        .tempdir_in(parent)
        .map_err(|e| format!("Impossible d'écrire à côté du programme : {e}"))?;
    let executable = directory.path().join("new-cpm.exe");
    let mut response = agent(120)
        .get(&asset.browser_download_url)
        .header("User-Agent", "CPM-Updater")
        .call()
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(MAX_DOWNLOAD + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 != asset.size || bytes.len() as u64 > MAX_DOWNLOAD {
        return Err("Taille du téléchargement incorrecte".into());
    }
    let expected = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
        .ok_or("Somme SHA-256 absente")?;
    if !verify_digest(&bytes, expected) {
        return Err("La somme SHA-256 du téléchargement ne correspond pas à GitHub".into());
    }
    if !bytes.starts_with(b"MZ") {
        return Err("Le fichier téléchargé n'est pas un exécutable Windows".into());
    }
    let mut file = std::fs::File::create(&executable).map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(PreparedUpdate {
        directory,
        executable,
        target,
    })
}

pub fn install(prepared: PreparedUpdate) -> Result<(), String> {
    if !cfg!(target_os = "windows") {
        return Err("Installation automatique disponible sur Windows uniquement".into());
    }
    let script = prepared.directory.path().join("apply-update.ps1");
    let backup = prepared.directory.path().join("previous-cpm.exe");
    std::fs::write(&script, include_str!("../assets/apply-update.ps1"))
        .map_err(|e| e.to_string())?;
    std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(&script)
        .arg("-OldPid")
        .arg(std::process::id().to_string())
        .arg("-TargetPath")
        .arg(&prepared.target)
        .arg("-DownloadPath")
        .arg(&prepared.executable)
        .arg("-BackupPath")
        .arg(&backup)
        .spawn()
        .map_err(|e| format!("Impossible de démarrer la mise à jour : {e}"))?;
    let _ = prepared.directory.keep();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn release(tag: &str) -> Release {
        Release {
            tag_name: tag.into(),
            html_url: format!("{PAGE_PREFIX}{tag}"),
            draft: false,
            prerelease: false,
            assets: Vec::new(),
        }
    }
    #[test]
    fn selects_newest_stable_rust_release_and_ignores_java() {
        let mut preview = release("rust-v9.0.0");
        preview.prerelease = true;
        let mut draft = release("rust-v8.0.0");
        draft.draft = true;
        let result = select_update(
            vec![
                release("Beta-0.8"),
                release("rust-v0.9.0"),
                release("rust-v0.10.0"),
                preview,
                draft,
                release("rust-v0.1.0"),
                release("rust-v1.0.0-beta.1"),
            ],
            &Version::parse("0.2.0").unwrap(),
        )
        .unwrap();
        assert_eq!(result.version, Version::parse("0.10.0").unwrap());
    }
    #[test]
    fn no_button_for_equal_older_or_invalid_release_urls() {
        let mut invalid = release("rust-v2.0.0");
        invalid.html_url = "https://example.test/release".into();
        assert!(select_update(
            vec![release("rust-v0.1.0"), release("rust-v0.2.0"), invalid],
            &Version::parse("0.2.0").unwrap()
        )
        .is_none());
    }
    #[test]
    fn asset_requires_trusted_url_size_and_sha256() {
        let bytes = b"MZ test executable";
        let hash = format!("{:x}", Sha256::digest(bytes));
        assert!(verify_digest(bytes, &hash));
        assert!(!verify_digest(b"tampered", &hash));
        let mut asset = Asset {
            name: platform_asset().into(),
            browser_download_url: format!("{DOWNLOAD_PREFIX}rust-v1.0.0/cpm-windows-x86_64.exe"),
            digest: Some(format!("sha256:{hash}")),
            size: bytes.len() as u64,
        };
        assert!(valid_asset(&asset));
        asset.browser_download_url = "http://example.test/update.exe".into();
        assert!(!valid_asset(&asset));
        asset.browser_download_url = format!("{DOWNLOAD_PREFIX}rust-v1.0.0/cpm-windows-x86_64.exe");
        asset.digest = None;
        assert!(!valid_asset(&asset));
    }
}
