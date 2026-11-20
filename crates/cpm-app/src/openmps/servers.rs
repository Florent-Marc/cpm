use super::{client, Client, OpenMps, Profile};
use eframe::egui;
use sha2::{Digest, Sha256};

pub(super) trait Vault {
    fn get(&self, account: &str) -> Result<Option<String>, String>;
    fn set(&self, account: &str, token: &str) -> Result<(), String>;
    fn delete(&self, account: &str) -> Result<(), String>;
}
pub(super) struct SystemVault;
impl Vault for SystemVault {
    fn get(&self, account: &str) -> Result<Option<String>, String> {
        match entry(account)?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(
                "Coffre système indisponible ou verrouillé. Saisir le jeton pour cette session."
                    .into(),
            ),
        }
    }
    fn set(&self, account: &str, token: &str) -> Result<(), String> {
        entry(account)?.set_password(token).map_err(|_| "Impossible d’enregistrer le jeton dans le coffre système. Vous pouvez décocher sa mémorisation et continuer pour cette session.".into())
    }
    fn delete(&self, account: &str) -> Result<(), String> {
        match entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err("Impossible de supprimer le jeton du coffre système. Déverrouiller le coffre puis réessayer.".into()),
        }
    }
}
fn entry(account: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new("fr.dynamx.cpm.openmps", account)
        .map_err(|_| "Coffre système indisponible".into())
}
fn account(profile: &Profile) -> String {
    // Bind credentials to the exact server, even when a profile is edited or renamed.
    let placeholder = format!("omps_{}", "x".repeat(43));
    let endpoint = Client::new(&profile.url, &placeholder)
        .expect("validated profile")
        .endpoint;
    format!("{}-{:x}", profile.id, Sha256::digest(endpoint.as_str()))
}
impl OpenMps {
    pub(super) fn reset_connection(&mut self) {
        self.caps = None;
        self.versions.clear();
        self.prepared = None;
        self.publication = None;
        self.recovered = None;
        self.open_request = None;
        self.confirm_publish = false;
        self.confirm_delete = false;
        self.status = "Non connecté — tester la connexion pour charger les packs".into();
        self.error = None;
    }
    fn stash_session(&mut self) {
        if let Some(profile) = self
            .profiles
            .iter()
            .find(|p| Some(&p.id) == self.selected.as_ref())
        {
            if profile.url == self.endpoint {
                if self.token.is_empty() {
                    self.sessions.remove(&account(profile));
                } else {
                    self.sessions.insert(account(profile), self.token.clone());
                }
            }
        }
    }
    pub(super) fn select_server(&mut self, id: &str, vault: &impl Vault) {
        if self.busy() {
            return;
        }
        let Some(profile) = self.profiles.iter().find(|p| p.id == id).cloned() else {
            return;
        };
        self.stash_session();
        self.reset_connection();
        self.selected = Some(profile.id.clone());
        self.profile_name = profile.name.clone();
        self.endpoint = profile.url.clone();
        self.remember_token = profile.remember_token;
        self.token = self
            .sessions
            .get(&account(&profile))
            .cloned()
            .unwrap_or_default();
        self.access_key = "legacy".into();
        if self.token.is_empty() && profile.remember_token {
            match vault.get(&account(&profile)) {
                Ok(Some(token)) => self.token = token,
                Ok(None) => {
                    self.status = "Jeton absent du coffre ; saisir à nouveau le jeton".into()
                }
                Err(error) => self.error = Some(error),
            }
        }
        for p in &mut self.profiles {
            p.active = p.id == id;
        }
    }
    fn new_server(&mut self) {
        self.stash_session();
        self.reset_connection();
        self.selected = None;
        self.profile_name.clear();
        self.endpoint.clear();
        self.token.clear();
        self.remember_token = false;
        self.access_key = "legacy".into();
    }
    fn save_server(&mut self, path: &std::path::Path, vault: &impl Vault) -> Result<(), String> {
        let placeholder = format!("omps_{}", "x".repeat(43));
        Client::new(&self.endpoint, &placeholder)?;
        if self.remember_token {
            Client::new(&self.endpoint, &self.token)?;
        }
        let old = self
            .profiles
            .iter()
            .find(|p| Some(&p.id) == self.selected.as_ref())
            .cloned();
        let id = old.as_ref().map(|p| p.id.clone()).unwrap_or_else(|| {
            format!(
                "{:x}",
                Sha256::digest(format!(
                    "{:?}-{}-{}",
                    std::time::SystemTime::now(),
                    std::process::id(),
                    self.profiles.len()
                ))
            )
        });
        let profile = Profile {
            id: id.clone(),
            name: self.profile_name.trim().into(),
            url: self.endpoint.trim().into(),
            remember_token: self.remember_token,
            active: true,
        };
        let mut profiles = self.profiles.clone();
        profiles.retain(|p| p.id != id);
        if profiles
            .iter()
            .any(|p| p.name.eq_ignore_ascii_case(&profile.name))
        {
            return Err("Un serveur porte déjà ce nom. Choisir un nom distinct.".into());
        }
        for p in &mut profiles {
            p.active = false;
        }
        profiles.push(profile.clone());
        client::validate_profiles(&profiles)?;
        let target = account(&profile);
        if profile.remember_token {
            vault.set(&target, &self.token)?;
        }
        if let Some(old) = &old {
            let previous = account(old);
            if old.remember_token && (previous != target || !profile.remember_token) {
                vault.delete(&previous)?;
            }
        }
        // Never persist a secret in JSON, including when the native vault fails.
        client::save_profiles(path, &profiles)?;
        self.profiles = profiles;
        self.selected = Some(id);
        self.profile_name = profile.name.clone();
        self.endpoint = profile.url.clone();
        if self.token.is_empty() {
            self.sessions.remove(&target);
        } else {
            self.sessions.insert(target.clone(), self.token.clone());
        }
        if let Some(old) = old {
            let previous = account(&old);
            if previous != target {
                self.sessions.remove(&previous);
            }
        }
        self.status = "Serveur enregistré".into();
        Ok(())
    }
    fn remove_server(&mut self, path: &std::path::Path, vault: &impl Vault) -> Result<(), String> {
        let Some(old) = self
            .profiles
            .iter()
            .find(|p| Some(&p.id) == self.selected.as_ref())
            .cloned()
        else {
            return Ok(());
        };
        if old.remember_token {
            vault.delete(&account(&old))?;
        }
        let profiles: Vec<_> = self
            .profiles
            .iter()
            .filter(|p| p.id != old.id)
            .cloned()
            .collect();
        client::save_profiles(path, &profiles)?;
        self.sessions.remove(&account(&old));
        self.profiles = profiles;
        self.new_server();
        self.status = "Serveur supprimé ; les packs distants sont conservés".into();
        Ok(())
    }
    pub(super) fn destination(&self) -> String {
        format!(
            "{} — {}",
            if self.profile_name.trim().is_empty() {
                "Serveur sans nom"
            } else {
                self.profile_name.trim()
            },
            self.endpoint.trim()
        )
    }
    pub(super) fn server_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_enabled_ui(!self.busy(), |ui| {
                let mut selected = None;
                for profile in &self.profiles { ui.horizontal(|ui| {
                    if ui.selectable_label(Some(&profile.id) == self.selected.as_ref(), &profile.name).clicked() { selected = Some(profile.id.clone()); }
                    ui.weak(&profile.url);
                }); }
                if let Some(id) = selected { self.select_server(&id, &SystemVault); self.persist_selection(); }
                if ui.button("+ Ajouter un serveur").clicked() { self.new_server(); }
                ui.separator();
                ui.horizontal(|ui| { ui.label("Nom"); ui.text_edit_singleline(&mut self.profile_name); });
                ui.horizontal(|ui| { ui.label("URL"); if ui.add(egui::TextEdit::singleline(&mut self.endpoint).hint_text("https://mps.exemple.fr/").desired_width(f32::INFINITY)).changed() {
                    self.token.clear(); self.remember_token = false; self.reset_connection();
                } });
                ui.horizontal(|ui| { ui.label("Jeton d’administration"); if ui.add(egui::TextEdit::singleline(&mut self.token).password(true).desired_width(f32::INFINITY)).changed() { self.reset_connection(); } });
                ui.checkbox(&mut self.remember_token, "Mémoriser le jeton dans le coffre sécurisé du système");
                ui.weak("Sinon, le jeton reste disponible uniquement pendant cette session. Aucun jeton n’est écrit dans le fichier des serveurs.");
                ui.horizontal(|ui| {
                    if ui.button("Enregistrer").clicked() { if let Err(e) = Self::profiles_path().and_then(|p| self.save_server(&p, &SystemVault)) { self.error = Some(e); } }
                    if ui.button("Tester la connexion").clicked() { self.connect_server(ctx); }
                    if ui.add_enabled(self.selected.is_some(), egui::Button::new("Supprimer…")).clicked() { self.confirm_delete = true; }
                });
                if self.confirm_delete {
                    ui.label("Supprimer cette connexion et son jeton mémorisé ? Les packs du serveur restent intacts.");
                    ui.horizontal(|ui| {
                        if ui.button("Confirmer la suppression").clicked() { self.confirm_delete = false; if let Err(e) = Self::profiles_path().and_then(|p| self.remove_server(&p, &SystemVault)) { self.error = Some(e); } }
                        if ui.button("Annuler").clicked() { self.confirm_delete = false; }
                    });
                }
            });
    }
    pub(super) fn persist_selection(&mut self) {
        if let Err(e) =
            Self::profiles_path().and_then(|p| client::save_profiles(&p, &self.profiles))
        {
            self.error = Some(e);
        }
    }
    pub(super) fn connect_server(&mut self, ctx: &egui::Context) {
        self.reset_connection();
        self.total = 0;
        match self.client() {
            Ok(client) => self.start(ctx, "Connexion et lecture des versions…", move || {
                let (caps, rows) = client.connect()?;
                Ok(super::Completed::Connected(caps, rows))
            }),
            Err(e) => self.error = Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::BTreeMap};
    #[derive(Default)]
    struct MemoryVault {
        secrets: RefCell<BTreeMap<String, String>>,
        fail: bool,
    }
    impl Vault for MemoryVault {
        fn get(&self, key: &str) -> Result<Option<String>, String> {
            if self.fail {
                return Err("Coffre verrouillé".into());
            }
            Ok(self.secrets.borrow().get(key).cloned())
        }
        fn set(&self, key: &str, value: &str) -> Result<(), String> {
            if self.fail {
                return Err("Coffre verrouillé".into());
            }
            self.secrets.borrow_mut().insert(key.into(), value.into());
            Ok(())
        }
        fn delete(&self, key: &str) -> Result<(), String> {
            if self.fail {
                return Err("Coffre verrouillé".into());
            }
            self.secrets.borrow_mut().remove(key);
            Ok(())
        }
    }
    fn token(c: &str) -> String {
        format!("omps_{}", c.repeat(43))
    }
    fn draft(app: &mut OpenMps, name: &str, url: &str, secret: &str) {
        app.new_server();
        app.profile_name = name.into();
        app.endpoint = url.into();
        app.token = secret.into();
    }
    #[test]
    fn multiple_servers_keep_tokens_and_remote_state_isolated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("servers.json");
        let vault = MemoryVault::default();
        let mut app = OpenMps::default();
        draft(&mut app, "Principal", "https://one.example/", &token("a"));
        app.remember_token = true;
        app.save_server(&path, &vault).unwrap();
        let first = app.selected.clone().unwrap();
        app.versions.push(client::Version {
            pack_key: "first-pack".into(),
            pack_version: "1".into(),
            client_sha256: None,
            client_size: 1,
            files_count: 1,
        });
        app.confirm_publish = true;
        draft(&mut app, "Tests", "https://two.example/", &token("b"));
        app.save_server(&path, &vault).unwrap();
        let second = app.selected.clone().unwrap();
        assert!(app.versions.is_empty());
        assert!(!app.confirm_publish);
        app.select_server(&first, &vault);
        assert_eq!(app.token, token("a"));
        assert!(app.versions.is_empty());
        assert!(app.caps.is_none());
        assert!(app.prepared.is_none());
        app.select_server(&second, &vault);
        assert_eq!(app.token, token("b"));
        let json = std::fs::read_to_string(&path).unwrap();
        assert!(!json.contains("omps_"));
        let mut restarted = OpenMps {
            profiles: client::load_profiles(&path).unwrap(),
            ..Default::default()
        };
        restarted.select_server(&first, &vault);
        assert_eq!(restarted.token, token("a"));
        restarted.select_server(&second, &vault);
        assert!(restarted.token.is_empty());
        // Switching is refused while an operation is in flight.
        let (_tx, rx) = std::sync::mpsc::channel();
        restarted.receiver = Some(rx);
        restarted.select_server(&first, &vault);
        assert_eq!(restarted.selected.as_ref(), Some(&second));
    }
    #[test]
    fn renaming_preserves_identity_changing_endpoint_never_reuses_saved_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("servers.json");
        let vault = MemoryVault::default();
        let mut app = OpenMps::default();
        draft(&mut app, "Principal", "https://one.example/", &token("a"));
        app.remember_token = true;
        app.save_server(&path, &vault).unwrap();
        let id = app.selected.clone().unwrap();
        let old_account = account(&app.profiles[0]);
        app.profile_name = "Production".into();
        app.save_server(&path, &vault).unwrap();
        assert_eq!(app.selected.as_ref(), Some(&id));
        assert_eq!(app.profiles.len(), 1);
        app.endpoint = "https://other.example/".into();
        app.token.clear();
        app.remember_token = false;
        app.reset_connection();
        app.save_server(&path, &vault).unwrap();
        assert!(!vault.secrets.borrow().contains_key(&old_account));
        app.select_server(&id, &vault);
        assert!(app.token.is_empty());
        app.remove_server(&path, &vault).unwrap();
        assert!(client::load_profiles(&path).unwrap().is_empty());
        assert!(app.selected.is_none());
    }
    #[test]
    fn locked_vault_never_falls_back_to_plaintext_and_profiles_survive_failed_save() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("servers.json");
        let vault = MemoryVault {
            fail: true,
            ..Default::default()
        };
        let mut app = OpenMps::default();
        draft(&mut app, "Principal", "https://one.example/", &token("a"));
        app.remember_token = true;
        assert!(app.save_server(&path, &vault).is_err());
        assert!(!path.exists());
        assert!(app.profiles.is_empty());
        app.remember_token = false;
        app.save_server(&path, &vault).unwrap();
        assert!(!std::fs::read_to_string(&path)
            .unwrap()
            .contains(&token("a")));
        let before = std::fs::read(&path).unwrap();
        draft(&mut app, "Principal", "https://two.example/", &token("b"));
        assert!(app.save_server(&path, &vault).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
    #[test]
    fn legacy_profiles_are_migrated_without_losing_servers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("servers.json");
        std::fs::write(&path,r#"[{"name":"Ancien","url":"https://old.example/"},{"name":"Autre","url":"https://other.example/"}]"#).unwrap();
        let profiles = client::load_profiles(&path).unwrap();
        assert_eq!(profiles.len(), 2);
        assert!(!profiles[0].id.is_empty());
        assert_ne!(profiles[0].id, profiles[1].id);
        assert!(!profiles[0].remember_token);
        client::save_profiles(&path, &profiles).unwrap();
        assert_eq!(client::load_profiles(&path).unwrap()[0].id, profiles[0].id);
    }
}
