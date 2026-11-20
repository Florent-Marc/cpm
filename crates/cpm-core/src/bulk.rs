use crate::{Document, Pack, ParameterCatalog, Severity};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BulkOperation {
    #[default]
    ReplaceExisting,
    AddMissing,
    Remove,
}
#[derive(Clone, Debug, Default)]
pub struct BulkRequest {
    pub paths: BTreeSet<PathBuf>,
    pub section: Option<String>,
    pub parameter: String,
    pub value: String,
    pub operation: BulkOperation,
}
pub struct BulkDocument {
    pub path: PathBuf,
    pub before: Document,
    pub after: Document,
}
pub struct BulkPlan {
    pub documents: Vec<BulkDocument>,
    pub warnings: Vec<String>,
}
impl Pack {
    pub fn plan_bulk_edit(
        &self,
        request: &BulkRequest,
        overrides: &BTreeMap<PathBuf, Document>,
        catalog: &ParameterCatalog,
    ) -> Result<BulkPlan, String> {
        if request.paths.is_empty() || request.paths.len() > 2000 {
            return Err("Sélectionner entre 1 et 2000 configurations".into());
        }
        let mut probe = Document::parse("").map_err(|e| e.to_string())?;
        probe
            .add_property(
                None,
                &request.parameter,
                if request.operation == BulkOperation::Remove {
                    ""
                } else {
                    &request.value
                },
            )
            .map_err(|e| e.to_string())?;
        let mut plan = BulkPlan {
            documents: Vec::new(),
            warnings: Vec::new(),
        };
        for path in &request.paths {
            let index = self
                .configs()
                .iter()
                .position(|p| p == path)
                .ok_or("Configuration inconnue dans la sélection")?;
            let before = overrides
                .get(path)
                .cloned()
                .map(Ok)
                .unwrap_or_else(|| self.read_config(index))?;
            let mut after = before.clone();
            let scopes: Vec<_> = if let Some(name) = &request.section {
                after
                    .sections()
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.name == *name)
                    .map(|(i, _)| Some(i))
                    .collect()
            } else {
                vec![None]
            };
            if scopes.is_empty() {
                plan.warnings.push(format!(
                    "{} : section absente, fichier ignoré",
                    path.display()
                ));
            }
            for scope in scopes {
                let definition = catalog
                    .parameters(path, &after, scope)
                    .into_iter()
                    .find(|d| {
                        d.name == request.parameter || d.aliases.contains(&request.parameter)
                    });
                let indices: Vec<_> = after
                    .properties()
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| {
                        p.section == scope
                            && (p.key == request.parameter
                                || definition
                                    .as_ref()
                                    .is_some_and(|d| p.key == d.name || d.aliases.contains(&p.key)))
                    })
                    .map(|(i, _)| i)
                    .collect();
                if (request.operation == BulkOperation::ReplaceExisting && !indices.is_empty())
                    || (request.operation == BulkOperation::AddMissing && indices.is_empty())
                {
                    if let Some(d) = &definition {
                        if !d.accepts_value(&request.value) {
                            return Err(format!(
                                "{} / {} : valeur incompatible avec {}",
                                path.display(),
                                request.parameter,
                                d.variable_type.label()
                            ));
                        }
                    }
                }
                match request.operation {
                    BulkOperation::ReplaceExisting => {
                        for index in indices {
                            after
                                .set_value(index, &request.value)
                                .map_err(|e| e.to_string())?;
                        }
                    }
                    BulkOperation::AddMissing if indices.is_empty() => {
                        after
                            .add_property(scope, &request.parameter, &request.value)
                            .map_err(|e| e.to_string())?;
                    }
                    BulkOperation::Remove => {
                        for index in indices.into_iter().rev() {
                            after.remove_property(index).map_err(|e| e.to_string())?;
                        }
                    }
                    _ => {}
                }
            }
            if before.source() != after.source() {
                for d in catalog
                    .diagnostics(path, &after)
                    .into_iter()
                    .filter(|d| d.severity == Severity::Error)
                {
                    plan.warnings.push(format!(
                        "{} / {} : {}",
                        path.display(),
                        d.parameter,
                        d.message
                    ));
                }
                plan.documents.push(BulkDocument {
                    path: path.clone(),
                    before,
                    after,
                });
            }
        }
        Ok(plan)
    }
}
