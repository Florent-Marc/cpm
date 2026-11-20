//! Review a frozen output against its source before writing a user destination.
use crate::{Document, ExportFormat, ExportOptions, Pack, ParameterCatalog};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Removed,
    Modified,
}
impl ChangeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Added => "Ajouté",
            Self::Removed => "Supprimé",
            Self::Modified => "Modifié",
        }
    }
}
#[derive(Clone, Debug)]
pub struct ValueChange {
    pub parameter: String,
    pub before: Option<String>,
    pub after: Option<String>,
}
#[derive(Clone, Debug)]
pub struct FileChange {
    pub path: PathBuf,
    pub kind: ChangeKind,
    pub parameters: Vec<ValueChange>,
    pub text_only: bool,
}
pub struct ExportReview {
    pub changes: Vec<FileChange>,
    pack: Pack,
    options: ExportOptions,
    catalog: ParameterCatalog,
    force: bool,
    _workspace: tempfile::TempDir,
}
impl ExportReview {
    pub fn export(self) -> Result<PathBuf, String> {
        self.pack
            .export_with_policy(&self.options, &BTreeMap::new(), &self.catalog, self.force)
    }
}
impl Pack {
    pub fn review_export(
        &self,
        options: &ExportOptions,
        documents: &BTreeMap<PathBuf, Document>,
        catalog: &ParameterCatalog,
        force: bool,
    ) -> Result<ExportReview, String> {
        let parent =
            fs::canonicalize(&options.destination).map_err(|e| format!("Destination : {e}"))?;
        let mut resolved = options.clone();
        resolved.destination = parent.clone();
        let target = resolved.target()?;
        if !parent.is_dir()
            || (self.source().is_dir() && parent.starts_with(self.source()))
            || (self.source().starts_with(&target) && self.source() != target)
        {
            return Err(
                "Choisir une sortie en dehors du pack source et de ses dossiers parents".into(),
            );
        }
        crate::export::check_existing(&target, &resolved)?;
        let workspace = tempfile::tempdir().map_err(|e| e.to_string())?;
        let mut snapshot = resolved.clone();
        snapshot.destination = workspace.path().into();
        snapshot.output_name = "review".into();
        snapshot.format = ExportFormat::Folder;
        snapshot.replace_existing = false;
        let output = self.export_with_policy(&snapshot, documents, catalog, force)?;
        let original = workspace.path().join("original");
        fs::create_dir(&original).map_err(|e| e.to_string())?;
        Pack::open(self.source())?.materialize(&original)?;
        let changes = compare_folders(&original, &output)?;
        fs::remove_dir_all(&original).map_err(|e| e.to_string())?;
        let pack = Pack::open(&output)?;
        Ok(ExportReview {
            changes,
            pack,
            options: resolved,
            catalog: catalog.clone(),
            force,
            _workspace: workspace,
        })
    }
}
fn files(root: &Path) -> Result<BTreeSet<PathBuf>, String> {
    let mut result = BTreeSet::new();
    let mut pending = vec![root.to_owned()];
    while let Some(dir) = pending.pop() {
        for e in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let e = e.map_err(|e| e.to_string())?;
            let kind = e.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() {
                pending.push(e.path());
            } else if kind.is_file() {
                result.insert(
                    e.path()
                        .strip_prefix(root)
                        .map_err(|e| e.to_string())?
                        .to_owned(),
                );
            } else {
                return Err("Type de fichier non pris en charge pendant la comparaison".into());
            }
        }
    }
    Ok(result)
}
fn equal(a: &Path, b: &Path) -> Result<bool, String> {
    if fs::metadata(a).map_err(|e| e.to_string())?.len()
        != fs::metadata(b).map_err(|e| e.to_string())?.len()
    {
        return Ok(false);
    }
    let mut a = fs::File::open(a).map_err(|e| e.to_string())?;
    let mut b = fs::File::open(b).map_err(|e| e.to_string())?;
    let mut x = [0u8; 65536];
    let mut y = [0u8; 65536];
    loop {
        let n = a.read(&mut x).map_err(|e| e.to_string())?;
        let m = b.read(&mut y).map_err(|e| e.to_string())?;
        if n != m || x[..n] != y[..m] {
            return Ok(false);
        }
        if n == 0 {
            return Ok(true);
        }
    }
}
fn values(doc: &Document) -> BTreeMap<String, String> {
    let mut counts = BTreeMap::new();
    let mut sections: Vec<String> = Vec::new();
    for section in doc.sections() {
        let parent = section
            .parent
            .map(|i| sections[i].clone())
            .unwrap_or_else(|| "Racine".to_owned());
        let key = format!("{parent}/{}", section.name);
        let count = counts.entry(key.clone()).or_insert(0);
        *count += 1;
        sections.push(format!("{key} #{}", count));
    }
    counts.clear();
    let mut result = BTreeMap::new();
    for p in doc.properties() {
        let scope = p.section.map(|i| sections[i].as_str()).unwrap_or("Racine");
        let key = format!("{scope}/{}", p.key);
        let count = counts.entry(key.clone()).or_insert(0);
        *count += 1;
        result.insert(format!("{key} #{}", count), p.value.clone());
    }
    result
}
/// Parameter-level changes, with section and duplicate occurrence identities.
pub fn parameter_changes(before: Option<&Document>, after: Option<&Document>) -> Vec<ValueChange> {
    let a = before.map(values).unwrap_or_default();
    let b = after.map(values).unwrap_or_default();
    a.keys()
        .chain(b.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|k| a.get(*k) != b.get(*k))
        .map(|k| ValueChange {
            parameter: k.clone(),
            before: a.get(k).cloned(),
            after: b.get(k).cloned(),
        })
        .collect()
}
fn config(path: &Path) -> Option<Document> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .ok()?
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 8 * 1024 * 1024 {
        return None;
    }
    Document::parse(std::str::from_utf8(&bytes).ok()?).ok()
}
fn compare_folders(before: &Path, after: &Path) -> Result<Vec<FileChange>, String> {
    let a = files(before)?;
    let b = files(after)?;
    let mut changes = Vec::new();
    for p in a.union(&b) {
        let kind = match (a.contains(p), b.contains(p)) {
            (true, true) if equal(&before.join(p), &after.join(p))? => continue,
            (true, true) => ChangeKind::Modified,
            (true, false) => ChangeKind::Removed,
            _ => ChangeKind::Added,
        };
        let is_config = p
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("dynx"));
        let parameters = if is_config {
            parameter_changes(
                config(&before.join(p)).as_ref(),
                config(&after.join(p)).as_ref(),
            )
        } else {
            Vec::new()
        };
        let text_only = is_config && parameters.is_empty();
        changes.push(FileChange {
            path: p.clone(),
            kind,
            parameters,
            text_only,
        });
    }
    Ok(changes)
}
