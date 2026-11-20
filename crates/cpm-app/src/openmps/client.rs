use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub remember_token: bool,
    #[serde(default)]
    pub active: bool,
}
#[derive(Clone, Deserialize)]
pub struct Capabilities {
    pub api_version: u32,
    pub protocol: String,
    pub public_url: String,
    pub mps_version: String,
    pub max_upload_bytes: u64,
    pub immutable_versions: bool,
    #[serde(default)]
    pub editable_download: bool,
    #[serde(default)]
    pub recovery_allowed: bool,
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
}
#[derive(Clone, Deserialize)]
pub struct Version {
    pub pack_key: String,
    pub pack_version: String,
    pub client_sha256: Option<String>,
    pub client_size: u64,
    pub files_count: usize,
}
#[derive(Clone, Deserialize)]
pub struct Publication {
    pub pack_key: String,
    pub pack_version: String,
    pub client_sha256: String,
    pub client_size: u64,
    pub files_count: usize,
    pub public_url: String,
    pub idempotent: bool,
    #[serde(default)]
    pub domains: Vec<String>,
}
#[derive(Clone)]
pub struct Client {
    pub(super) endpoint: url::Url,
    token: String,
    agent: ureq::Agent,
}
impl Client {
    pub fn new(base: &str, token: &str) -> Result<Self, String> {
        let mut endpoint = url::Url::parse(base.trim()).map_err(|_| "URL OpenMPS invalide")?;
        if !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            return Err("Utiliser une URL sans identifiants, paramètres ou fragment".into());
        }
        let loopback = match endpoint.host() {
            Some(url::Host::Domain(d)) => d.eq_ignore_ascii_case("localhost"),
            Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
            Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
            _ => false,
        };
        if endpoint.scheme() != "https" && !(endpoint.scheme() == "http" && loopback) {
            return Err("La connexion d’administration nécessite HTTPS (HTTP autorisé uniquement sur localhost pour les tests)".into());
        }
        if endpoint.path().ends_with("/api/v1.php") {
        } else {
            let path = format!("{}/api/v1.php", endpoint.path().trim_end_matches('/'));
            endpoint.set_path(&path);
        }
        let token = token.trim();
        if !token.starts_with("omps_")
            || token.len() != 48
            || !token[5..]
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
        {
            return Err(
                "Coller un jeton d’administration OpenMPS valide, créé avec tools/api_token.php"
                    .into(),
            );
        }
        let agent = ureq::Agent::config_builder()
            .max_redirects(0)
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(10)))
            .timeout_global(Some(Duration::from_secs(600)))
            .build()
            .into();
        Ok(Self {
            endpoint,
            token: token.into(),
            agent,
        })
    }
    fn url(&self, action: &str) -> url::Url {
        let mut url = self.endpoint.clone();
        url.query_pairs_mut().append_pair("action", action);
        url
    }
    fn redacted(&self, error: impl std::fmt::Display) -> String {
        error.to_string().replace(&self.token, "[jeton masqué]")
    }
    fn json(
        &self,
        mut response: ureq::http::Response<ureq::Body>,
    ) -> Result<serde_json::Value, String> {
        let status = response.status();
        let body = response
            .body_mut()
            .with_config()
            .limit(1024 * 1024)
            .read_to_string()
            .map_err(|e| self.redacted(e))?;
        let json:serde_json::Value=serde_json::from_str(&body).map_err(|_|format!("Le serveur n’a pas renvoyé l’API OpenMPS v1 (HTTP {status}). Installer ou mettre à jour l’API avant de publier."))?;
        if !status.is_success() || json["success"] != true {
            return Err(self.redacted(format!(
                "HTTP {status} : {}",
                json["error"].as_str().unwrap_or("Requête OpenMPS refusée")
            )));
        }
        Ok(json)
    }
    pub fn connect(&self) -> Result<(Capabilities, Vec<Version>), String> {
        let response = self
            .agent
            .get(self.url("capabilities").as_str())
            .header("Authorization", format!("Bearer {}", self.token))
            .call()
            .map_err(|e| self.redacted(e))?;
        let caps: Capabilities =
            serde_json::from_value(self.json(response)?).map_err(|e| e.to_string())?;
        if caps.api_version != 1
            || caps.protocol != "openmps-admin/1"
            || !caps.immutable_versions
            || caps.mps_version != "1.3.0"
        {
            return Err("Version ou capacités de l’API OpenMPS non prises en charge".into());
        }
        let public =
            url::Url::parse(&caps.public_url).map_err(|_| "URL publique OpenMPS invalide")?;
        if !["http", "https"].contains(&public.scheme())
            || !public.username().is_empty()
            || public.password().is_some()
        {
            return Err("URL publique OpenMPS invalide".into());
        }
        let response = self
            .agent
            .get(self.url("packs").as_str())
            .header("Authorization", format!("Bearer {}", self.token))
            .call()
            .map_err(|e| self.redacted(e))?;
        let rows = self.json(response)?;
        let versions = serde_json::from_value(rows["packs"].clone()).map_err(|e| e.to_string())?;
        Ok((caps, versions))
    }
    pub fn publish(
        &self,
        prepared: &cpm_core::PreparedOpenMps,
        access_key: &str,
        progress: Arc<AtomicU64>,
    ) -> Result<Publication, String> {
        if access_key.is_empty()
            || access_key.len() > 1024
            || !access_key.is_ascii()
            || access_key.chars().any(char::is_control)
        {
            return Err("Clé MPS du jeu invalide".into());
        }
        let digest = sha256(&prepared.archive)?;
        let file = File::open(&prepared.archive).map_err(|e| e.to_string())?;
        let size = file.metadata().map_err(|e| e.to_string())?.len();
        let mut reader = ProgressReader { file, progress };
        let body = ureq::SendBody::from_reader(&mut reader);
        let response = self
            .agent
            .post(self.url("publish").as_str())
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/zip")
            .header("Content-Length", size.to_string())
            .header("X-OpenMPS-Pack-Key", &prepared.key)
            .header("X-OpenMPS-Pack-Version", &prepared.version)
            .header("X-OpenMPS-SHA256", digest)
            .header("X-OpenMPS-Access-Key", access_key)
            .send(body)
            .map_err(|e| self.redacted(e))?;
        let json = self
            .json(response)
            .map_err(|e| e.replace(access_key, "[clé MPS masquée]"))?;
        let publication: Publication =
            serde_json::from_value(json["publication"].clone()).map_err(|e| e.to_string())?;
        if publication.pack_key != prepared.key
            || publication.pack_version != prepared.version
            || !valid_sha(&publication.client_sha256)
        {
            return Err("Le reçu OpenMPS ne correspond pas à la publication demandée ; vérifier les versions sur le serveur avant de réessayer".into());
        }
        Ok(publication)
    }
    pub fn recover(
        &self,
        version: &Version,
        parent: &Path,
        progress: Arc<AtomicU64>,
    ) -> Result<std::path::PathBuf, String> {
        cpm_core::validate_mps_key(&version.pack_key)?;
        cpm_core::validate_mps_version(&version.pack_version)?;
        let mut url = self.url("source");
        url.query_pairs_mut()
            .append_pair("pack_key", &version.pack_key)
            .append_pair("pack_version", &version.pack_version);
        let mut response = self
            .agent
            .get(url.as_str())
            .header("Authorization", format!("Bearer {}", self.token))
            .call()
            .map_err(|e| self.redacted(e))?;
        if !response.status().is_success() {
            self.json(response)?;
            return Err("Récupération refusée".into());
        }
        for (name, expected) in [
            ("X-OpenMPS-Pack-Key", version.pack_key.as_str()),
            ("X-OpenMPS-Pack-Version", version.pack_version.as_str()),
        ] {
            if response.headers().get(name).and_then(|h| h.to_str().ok()) != Some(expected) {
                return Err("La copie distante ne correspond pas au pack demandé".into());
            }
        }
        let expected = response
            .headers()
            .get("X-OpenMPS-SHA256")
            .and_then(|h| h.to_str().ok())
            .filter(|h| valid_sha(h))
            .ok_or("Empreinte de récupération absente")?
            .to_owned();
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        let mut reader = response.body_mut().as_reader().take(512 * 1024 * 1024 + 1);
        let mut bytes = [0u8; 65536];
        let mut total = 0u64;
        loop {
            let n = reader.read(&mut bytes).map_err(|e| self.redacted(e))?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > 512 * 1024 * 1024 {
                return Err("Copie éditable supérieure à 512 Mio".into());
            }
            temp.write_all(&bytes[..n]).map_err(|e| e.to_string())?;
            progress.store(total, Ordering::Relaxed);
        }
        temp.flush().map_err(|e| e.to_string())?;
        temp.as_file().sync_all().map_err(|e| e.to_string())?;
        if sha256(temp.path())? != expected {
            return Err("L’empreinte de la copie éditable ne correspond pas au serveur".into());
        }
        cpm_core::recover_pack_archive(
            temp.path(),
            parent,
            &version.pack_key,
            &version.pack_version,
        )
    }
    pub fn download(
        &self,
        version: &Version,
        path: &Path,
        progress: Arc<AtomicU64>,
    ) -> Result<(), String> {
        cpm_core::validate_mps_key(&version.pack_key)?;
        cpm_core::validate_mps_version(&version.pack_version)?;
        if fs::symlink_metadata(path).is_ok() {
            return Err("Le fichier de destination existe déjà ; choisir un nouveau nom".into());
        }
        let mut url = self.url("client");
        url.query_pairs_mut()
            .append_pair("pack_key", &version.pack_key)
            .append_pair("pack_version", &version.pack_version);
        let mut response = self
            .agent
            .get(url.as_str())
            .header("Authorization", format!("Bearer {}", self.token))
            .call()
            .map_err(|e| self.redacted(e))?;
        if !response.status().is_success() {
            self.json(response)?;
            return Err("Téléchargement refusé".into());
        }
        let expected = response
            .headers()
            .get("X-OpenMPS-SHA256")
            .and_then(|h| h.to_str().ok())
            .filter(|h| valid_sha(h))
            .ok_or("Le serveur n’a pas fourni l’empreinte du ZIP client")?
            .to_owned();
        if version
            .client_sha256
            .as_deref()
            .is_some_and(|h| h != expected)
        {
            return Err(
                "Le ZIP client a changé depuis la lecture des versions ; rafraîchir la liste"
                    .into(),
            );
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        let mut reader = response.body_mut().as_reader().take(512 * 1024 * 1024 + 1);
        let mut buffer = [0; 65536];
        let mut total = 0;
        loop {
            let count = reader.read(&mut buffer).map_err(|e| self.redacted(e))?;
            if count == 0 {
                break;
            }
            total += count as u64;
            if total > 512 * 1024 * 1024 {
                return Err("ZIP client supérieur à 512 Mio".into());
            }
            temp.write_all(&buffer[..count])
                .map_err(|e| e.to_string())?;
            progress.store(total, Ordering::Relaxed);
        }
        temp.flush().map_err(|e| e.to_string())?;
        temp.as_file().sync_all().map_err(|e| e.to_string())?;
        if sha256(temp.path())? != expected {
            return Err("L’empreinte du ZIP client ne correspond pas au serveur".into());
        }
        cpm_core::validate_mps_client_archive(
            temp.path(),
            &version.pack_key,
            &version.pack_version,
        )?;
        temp.persist_noclobber(path)
            .map_err(|e| e.error.to_string())?;
        Ok(())
    }
}
fn valid_sha(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
pub fn sha256(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
struct ProgressReader {
    file: File,
    progress: Arc<AtomicU64>,
}
impl Read for ProgressReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = self.file.read(buffer)?;
        self.progress.fetch_add(count as u64, Ordering::Relaxed);
        Ok(count)
    }
}
pub fn load_profiles(path: &Path) -> Result<Vec<Profile>, String> {
    match fs::read(path) {
        Ok(bytes) => {
            if bytes.len() > 1024 * 1024 {
                return Err("Fichier de profils OpenMPS trop volumineux".into());
            }
            let mut profiles: Vec<Profile> =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            for profile in &mut profiles {
                if profile.id.is_empty() {
                    profile.id = format!(
                        "{:x}",
                        Sha256::digest(format!("{}\n{}", profile.name, profile.url))
                    );
                }
            }
            validate_profiles(&profiles)?;
            Ok(profiles)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.to_string()),
    }
}
pub(super) fn validate_profiles(profiles: &[Profile]) -> Result<(), String> {
    if profiles.len() > 64 {
        return Err("Maximum 64 profils OpenMPS".into());
    }
    let placeholder = format!("omps_{}", "x".repeat(43));
    for profile in profiles {
        if profile.name.trim().is_empty() || profile.name.len() > 160 {
            return Err("Nom du profil invalide".into());
        }
        Client::new(&profile.url, &placeholder)?;
    }
    let mut ids = std::collections::BTreeSet::new();
    for profile in profiles {
        if profile.id.is_empty() || profile.id.len() > 128 || !ids.insert(&profile.id) {
            return Err("Identifiants de serveurs invalides ou dupliqués".into());
        }
    }
    if profiles.iter().filter(|p| p.active).count() > 1 {
        return Err("Un seul serveur actif est autorisé".into());
    }
    Ok(())
}
pub fn save_profiles(path: &Path, profiles: &[Profile]) -> Result<(), String> {
    validate_profiles(profiles)?;
    let parent = path.parent().ok_or("Dossier de profils absent")?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut tmp, profiles).map_err(|e| e.to_string())?;
    tmp.flush().map_err(|e| e.to_string())?;
    tmp.persist(path).map_err(|e| e.error.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_server_urls_and_never_persists_administration_tokens() {
        let token = format!("omps_{}", "x".repeat(43));
        assert!(Client::new("http://127.0.0.1:8081/", &token).is_ok());
        assert!(Client::new("https://example.com/subfolder/", &token).is_ok());
        assert!(Client::new("http://example.com/", &token).is_err());
        assert!(Client::new("https://user:pass@example.com/", &token).is_err());
        assert!(Client::new("https://example.com/?token=x", &token).is_err());
        assert!(Client::new("https://example.com/", "legacy").is_err());
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("profiles.json");
        save_profiles(
            &path,
            &[Profile {
                id: "test".into(),
                remember_token: false,
                active: true,
                name: "Test".into(),
                url: "https://example.com/".into(),
            }],
        )
        .unwrap();
        assert_eq!(load_profiles(&path).unwrap().len(), 1);
        assert!(!fs::read_to_string(&path).unwrap().contains(&token));
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    #[test]
    #[ignore = "Run with the loopback PHP fixture in integrations/openmps/test_roundtrip.py"]
    fn publishes_and_downloads_against_real_openmps_api() {
        let url = std::env::var("CPM_OPENMPS_TEST_URL").expect("loopback test URL");
        let token = std::env::var("CPM_OPENMPS_TEST_TOKEN").expect("synthetic test token");
        let client = Client::new(&url, &token).unwrap();
        let (caps, _) = client.connect().unwrap();
        assert_eq!(caps.api_version, 1);
        let source = tempfile::tempdir().unwrap();
        std::fs::write(
            source.path().join("pack_info.dynx"),
            "PackName: Original\nPackVersion: 1.0\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        std::fs::write(source.path().join("vehicle_car.dynx"),"Name: Car\nDescription: Example\nModel: car.obj\nEmptyMass: 1000\nDragCoefficient: 0.3\nCenterOfGravityOffset: 0 0 0\n").unwrap();
        std::fs::write(
            source.path().join("car.obj"),
            "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
        )
        .unwrap();
        let pack = cpm_core::Pack::open(source.path()).unwrap();
        let prepared = pack
            .prepare_openmps(
                "cpm_contract",
                "2.0",
                &std::collections::BTreeMap::new(),
                &cpm_core::ParameterCatalog::builtin(),
            )
            .unwrap();
        let upload = Arc::new(AtomicU64::new(0));
        let receipt = client.publish(&prepared, "legacy", upload.clone()).unwrap();
        assert!(!receipt.idempotent);
        assert_eq!(
            upload.load(Ordering::Relaxed),
            std::fs::metadata(&prepared.archive).unwrap().len()
        );
        assert!(
            client
                .publish(&prepared, "legacy", Arc::new(AtomicU64::new(0)))
                .unwrap()
                .idempotent
        );
        let (_, versions) = client.connect().unwrap();
        let version = versions
            .iter()
            .find(|v| v.pack_key == "cpm_contract" && v.pack_version == "2.0")
            .unwrap();
        let destination = tempfile::tempdir().unwrap();
        let path = destination.path().join("players.zip");
        client
            .download(version, &path, Arc::new(AtomicU64::new(0)))
            .unwrap();
        assert_eq!(sha256(&path).unwrap(), receipt.client_sha256);
        assert!(client
            .download(version, &path, Arc::new(AtomicU64::new(0)))
            .is_err());
        assert!(caps.editable_download && caps.recovery_allowed);
        let recovered = client
            .recover(version, destination.path(), Arc::new(AtomicU64::new(0)))
            .unwrap();
        let restored = cpm_core::Pack::open(&recovered).unwrap();
        assert!(restored.contains_resource(std::path::Path::new("assets/cpm_contract/car.obj")));
        let second = client
            .recover(version, destination.path(), Arc::new(AtomicU64::new(0)))
            .unwrap();
        assert_ne!(recovered, second);
        assert!(!source.path().join("assets").exists());
    }
}
