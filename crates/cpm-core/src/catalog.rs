//! Définitions portables des paramètres, séparées du contenu des packs.
use crate::{parameter_catalog, Document, ParameterSpec};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VariableType {
    #[default]
    Text,
    Integer,
    Decimal,
    Boolean,
    Vector2,
    Vector3,
    Quaternion,
    Resource,
    Enumeration,
    List,
}
impl VariableType {
    pub const ALL: [Self; 10] = [
        Self::Text,
        Self::Integer,
        Self::Decimal,
        Self::Boolean,
        Self::Vector2,
        Self::Vector3,
        Self::Quaternion,
        Self::Resource,
        Self::Enumeration,
        Self::List,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "Texte",
            Self::Integer => "Entier",
            Self::Decimal => "Décimal",
            Self::Boolean => "Booléen",
            Self::Vector2 => "Vecteur / paire (2)",
            Self::Vector3 => "Vecteur (3)",
            Self::Quaternion => "Quaternion (4)",
            Self::Resource => "Ressource",
            Self::Enumeration => "Choix",
            Self::List => "Liste",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Ignore,
    Warning,
    #[default]
    Error,
}
impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ignore => "Ignoré",
            Self::Warning => "Avertissement",
            Self::Error => "Erreur (bloque l'export)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterDefinition {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub variable_type: VariableType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub severity: Severity,
    #[serde(default)]
    pub default_value: Option<String>,
    #[serde(default)]
    pub allow_empty: bool,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub choices: Vec<String>,
}
impl Default for ParameterDefinition {
    fn default() -> Self {
        Self {
            name: "NewParameter".into(),
            description: String::new(),
            variable_type: VariableType::Text,
            required: false,
            severity: Severity::Error,
            default_value: None,
            allow_empty: true,
            aliases: Vec::new(),
            choices: Vec::new(),
        }
    }
}
impl ParameterDefinition {
    pub fn is_present(&self, document: &Document, section: Option<usize>) -> bool {
        document
            .properties()
            .iter()
            .any(|p| p.section == section && (p.key == self.name || self.aliases.contains(&p.key)))
    }
    pub fn accepts_value(&self, value: &str) -> bool {
        self.valid_value(value)
    }
    fn valid_value(&self, value: &str) -> bool {
        let value = value.trim();
        if value.is_empty() {
            return self.allow_empty;
        }
        let number = |s: &str| s.parse::<f64>().is_ok_and(f64::is_finite);
        let vector = |size| {
            let values: Vec<_> = value.split_whitespace().collect();
            values.len() == size && values.iter().all(|v| number(v))
        };
        match self.variable_type {
            VariableType::Text | VariableType::List => true,
            VariableType::Integer => value.parse::<i64>().is_ok(),
            VariableType::Decimal => {
                number(value)
                    || (self.name == "MaxVehicleSpeed" && value.eq_ignore_ascii_case("infinite"))
            }
            VariableType::Boolean => {
                value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false")
            }
            VariableType::Vector2 => vector(2),
            VariableType::Vector3 => vector(3),
            VariableType::Quaternion => vector(4),
            VariableType::Resource => !value.chars().any(char::is_whitespace),
            VariableType::Enumeration => self
                .choices
                .iter()
                .any(|choice| choice.eq_ignore_ascii_case(value)),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogRule {
    pub file_pattern: String,
    #[serde(default)]
    pub section_pattern: Option<String>,
    #[serde(default)]
    pub parent_pattern: Option<String>,
    pub parameters: Vec<ParameterDefinition>,
}
impl Default for CatalogRule {
    fn default() -> Self {
        Self {
            file_pattern: "addon_*.dynx".into(),
            section_pattern: None,
            parent_pattern: None,
            parameters: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogFile {
    pub schema_version: u32,
    pub label: String,
    pub rules: Vec<CatalogRule>,
}
#[derive(Debug, Clone)]
pub struct NamedCatalogFile {
    pub filename: String,
    pub catalog: CatalogFile,
}
#[derive(Debug, Clone)]
pub struct ParameterCatalog {
    pub files: Vec<NamedCatalogFile>,
}
impl Default for ParameterCatalog {
    fn default() -> Self {
        Self::builtin()
    }
}
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub section: Option<usize>,
    pub parameter: String,
    pub severity: Severity,
    pub message: String,
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.parameter, self.message)
    }
}
fn matches(pattern: &str, value: &str, case_sensitive: bool) -> bool {
    glob::Pattern::new(pattern).is_ok_and(|p| {
        p.matches_with(
            value,
            glob::MatchOptions {
                case_sensitive,
                ..Default::default()
            },
        )
    })
}
impl ParameterCatalog {
    pub fn parameters(
        &self,
        path: &Path,
        document: &Document,
        section: Option<usize>,
    ) -> Vec<ParameterDefinition> {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let mut result = BTreeMap::new();
        for file in &self.files {
            for rule in &file.catalog.rules {
                if !matches(&rule.file_pattern, &name, false) {
                    continue;
                }
                let scope = match (&rule.section_pattern, section) {
                    (None, None) => rule.parent_pattern.is_none(),
                    (Some(pattern), Some(id)) => document.sections().get(id).is_some_and(|s| {
                        matches(pattern, &s.name, true)
                            && rule.parent_pattern.as_ref().is_none_or(|p| {
                                s.parent.is_some_and(|parent| {
                                    matches(p, &document.sections()[parent].name, true)
                                })
                            })
                    }),
                    _ => false,
                };
                if scope {
                    for parameter in &rule.parameters {
                        result.insert(parameter.name.clone(), parameter.clone());
                    }
                }
            }
        }
        result.into_values().collect()
    }
    pub fn diagnostics(&self, path: &Path, document: &Document) -> Vec<Diagnostic> {
        let mut result = Vec::new();
        for section in std::iter::once(None).chain((0..document.sections().len()).map(Some)) {
            for definition in self.parameters(path, document, section) {
                if definition.severity == Severity::Ignore {
                    continue;
                }
                let values: Vec<_> = document
                    .properties()
                    .iter()
                    .filter(|p| {
                        p.section == section
                            && (p.key == definition.name || definition.aliases.contains(&p.key))
                    })
                    .collect();
                let problem = if values.is_empty() && definition.required {
                    Some("paramètre obligatoire absent".to_owned())
                } else if values.len() > 1 && definition.required {
                    Some("paramètre obligatoire dupliqué".to_owned())
                } else if values.iter().any(|p| !definition.valid_value(&p.value)) {
                    Some(format!(
                        "valeur invalide, type attendu : {}",
                        definition.variable_type.label()
                    ))
                } else {
                    None
                };
                if let Some(message) = problem {
                    result.push(Diagnostic {
                        section,
                        parameter: definition.name,
                        severity: definition.severity,
                        message,
                    });
                }
            }
        }
        result
    }
    pub fn errors(&self, path: &Path, document: &Document) -> Vec<String> {
        self.diagnostics(path, document)
            .into_iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| {
                let context = d
                    .section
                    .map(|id| document.sections()[id].name.as_str())
                    .unwrap_or("Racine");
                format!("{context} / {d}")
            })
            .collect()
    }
    pub fn builtin() -> Self {
        let mut rules = Vec::new();
        let empty = Document::parse("").expect("document vide");
        for (pattern, sample) in [
            ("pack_info.dynx", "pack_info.dynx"),
            ("engine_*.dynx", "engine_demo.dynx"),
            ("wheel_*.dynx", "wheel_demo.dynx"),
            ("vehicle_*.dynx", "vehicle_demo.dynx"),
            ("trailer_*.dynx", "trailer_demo.dynx"),
            ("boat_*.dynx", "boat_demo.dynx"),
            ("helicopter_*.dynx", "helicopter_demo.dynx"),
            ("item_*.dynx", "item_demo.dynx"),
            ("armor_*.dynx", "armor_demo.dynx"),
            ("block_*.dynx", "block_demo.dynx"),
            ("prop_*.dynx", "prop_demo.dynx"),
        ] {
            rules.push(CatalogRule {
                file_pattern: pattern.into(),
                section_pattern: None,
                parent_pattern: None,
                parameters: parameter_catalog(Path::new(sample), &empty, None)
                    .into_iter()
                    .map(definition)
                    .collect(),
            });
        }
        let mut add = |file: &str, sample: &str, pattern: &str, section_name: &str| {
            let doc = Document::parse(&format!("{section_name}{{\n}}\n")).expect("section exemple");
            rules.push(CatalogRule {
                file_pattern: file.into(),
                section_pattern: Some(pattern.into()),
                parent_pattern: None,
                parameters: parameter_catalog(Path::new(sample), &doc, Some(0))
                    .into_iter()
                    .map(definition)
                    .collect(),
            });
        };
        add(
            "pack_info.dynx",
            "pack_info.dynx",
            "RequiredAddon",
            "RequiredAddon",
        );
        add("engine_*.dynx", "engine_demo.dynx", "Point_*", "Point_0");
        add("engine_*.dynx", "engine_demo.dynx", "Gear_*", "Gear_0");
        for kind in ["vehicle", "trailer", "boat", "helicopter"] {
            for (pattern, name) in [
                ("Shape*", "Shape_demo"),
                ("SteeringWheel*", "SteeringWheel"),
                ("ForcePoint*", "ForcePoint_demo"),
                ("Wheel*", "Wheel_demo"),
                ("Seat*", "Seat_demo"),
                ("*Seat", "DriverSeat"),
            ] {
                add(
                    &format!("{kind}_*.dynx"),
                    &format!("{kind}_demo.dynx"),
                    pattern,
                    name,
                );
            }
        }
        for kind in ["sound", "sounds"] {
            for parent in ["Interior", "Exterior"] {
                let doc =
                    Document::parse(&format!("Engine{{\n{parent}{{\nStarting{{\n}}\n}}\n}}\n"))
                        .expect("sons exemple");
                rules.push(CatalogRule {
                    file_pattern: format!("{kind}_*.dynx"),
                    section_pattern: Some("*".into()),
                    parent_pattern: Some(parent.into()),
                    parameters: parameter_catalog(
                        Path::new(&format!("{kind}_demo.dynx")),
                        &doc,
                        Some(2),
                    )
                    .into_iter()
                    .map(definition)
                    .collect(),
                });
            }
        }
        Self {
            files: vec![
                NamedCatalogFile {
                    filename: "dynamx.json".into(),
                    catalog: CatalogFile {
                        schema_version: 1,
                        label: "DynamX".into(),
                        rules,
                    },
                },
                NamedCatalogFile {
                    filename: "basics-addon.json".into(),
                    catalog: serde_json::from_str(include_str!("../catalogs/basics-addon.json"))
                        .expect("catalogue BasicsAddon valide"),
                },
            ],
        }
    }
}
fn definition(spec: ParameterSpec) -> ParameterDefinition {
    let key = spec.key;
    let choices = match key {
        "Item3DRenderLocation" => vec!["NONE", "WORLD", "ALL"],
        "PlayerStandOnTop" => vec!["ALWAYS", "PROGRESSIVE", "NEVER"],
        "PlayerPosition" => vec!["LYING", "SITTING", "STANDING"],
        _ => Vec::new(),
    };
    let variable_type = if !choices.is_empty() {
        VariableType::Enumeration
    } else if [
        "Driver",
        "DrivingWheel",
        "IsRight",
        "IsSteerable",
        "HandBrakingWheel",
        "ShouldLimitFieldOfView",
        "UseComplexCollision",
        "UseComplexCollisions",
    ]
    .contains(&key)
    {
        VariableType::Boolean
    } else if [
        "EmptyMass",
        "DefaultZoomLevel",
        "SteeringMethod",
        "MaxItemStackSize",
        "Durability",
        "Enchantability",
    ]
    .contains(&key)
    {
        VariableType::Integer
    } else if [
        "Position",
        "Scale",
        "ScaleModifier",
        "CenterOfGravityOffset",
        "Intensity",
        "RotationPoint",
        "PlayerSize",
        "ItemRotate",
        "ItemTranslate",
        "Rotate",
        "Translate",
    ]
    .contains(&key)
    {
        VariableType::Vector3
    } else if ["Rotation", "SuspensionAxis"].contains(&key) {
        VariableType::Quaternion
    } else if ["RPMPower", "RPMRange", "SpeedRange", "PitchRange"].contains(&key) {
        VariableType::Vector2
    } else if key == "Model" {
        VariableType::Resource
    } else if [
        "Textures",
        "DamageReduction",
        "ArmorArms",
        "ArmorFoot",
        "ArmorLegs",
    ]
    .contains(&key)
    {
        VariableType::List
    } else if [
        "Power",
        "MaxRPM",
        "Braking",
        "TurnSpeed",
        "BrakeForce",
        "Friction",
        "Radius",
        "RimRadius",
        "RollInInfluence",
        "SuspensionMaxForce",
        "SuspensionRestLength",
        "SuspensionStiffness",
        "WheelDampingRelaxation",
        "WheelsDampingCompression",
        "Width",
        "HandBrakeForce",
        "DragCoefficient",
        "AngularDamping",
        "LinearDamping",
        "MaxVehicleSpeed",
        "ShapeYOffset",
        "MaxTurn",
        "CameraPositionY",
        "CameraRotation",
        "MaxPitch",
        "MaxYaw",
        "MinPitch",
        "MinYaw",
        "ItemScale",
        "LightLevel",
        "RenderDistanceSquared",
        "DespawnTime",
    ]
    .contains(&key)
    {
        VariableType::Decimal
    } else {
        VariableType::Text
    };
    ParameterDefinition {
        name: key.into(),
        description: builtin_description(key).into(),
        variable_type,
        required: spec.required,
        severity: Severity::Error,
        default_value: (!spec.default_value.is_empty()).then(|| spec.default_value.to_owned()),
        allow_empty: spec.allow_empty,
        aliases: spec.aliases.iter().map(|s| (*s).into()).collect(),
        choices: choices.into_iter().map(str::to_owned).collect(),
    }
}

pub fn validate_catalog_file(file: &CatalogFile) -> Result<(), String> {
    if file.schema_version != 1 {
        return Err("Version de format de catalogue non prise en charge".into());
    }
    for rule in &file.rules {
        if rule.file_pattern.is_empty() {
            return Err("Le motif de fichier est vide".into());
        }
        for pattern in std::iter::once(&rule.file_pattern)
            .chain(rule.section_pattern.iter())
            .chain(rule.parent_pattern.iter())
        {
            if pattern.is_empty() {
                return Err("Motif vide".into());
            }
            glob::Pattern::new(pattern).map_err(|e| e.to_string())?;
        }
        if rule.section_pattern.is_none() && rule.parent_pattern.is_some() {
            return Err("Un parent ne peut pas être défini pour la racine".into());
        }
        let mut names = BTreeSet::new();
        for parameter in &rule.parameters {
            for name in std::iter::once(&parameter.name).chain(parameter.aliases.iter()) {
                if name.is_empty()
                    || !name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c))
                {
                    return Err(format!("Nom de paramètre invalide : {name}"));
                }
                if !names.insert(name) {
                    return Err(format!("Paramètre ou alias dupliqué : {name}"));
                }
            }
            if parameter.variable_type == VariableType::Enumeration
                && (parameter.choices.is_empty()
                    || parameter.choices.iter().any(|s| s.trim().is_empty()))
            {
                return Err(format!(
                    "{} : renseigner les choix autorisés",
                    parameter.name
                ));
            }
            if let Some(value) = &parameter.default_value {
                if value.contains(['\r', '\n']) || !parameter.valid_value(value) {
                    return Err(format!(
                        "{} : valeur par défaut incompatible avec le type",
                        parameter.name
                    ));
                }
            }
        }
    }
    Ok(())
}

pub struct CatalogStore {
    pub directory: PathBuf,
}
pub struct CatalogLoad {
    pub catalog: ParameterCatalog,
    pub warnings: Vec<String>,
}
impl CatalogStore {
    pub fn load(&self) -> Result<CatalogLoad, String> {
        fs::create_dir_all(&self.directory)
            .map_err(|e| format!("{} : {e}", self.directory.display()))?;
        let builtin = ParameterCatalog::builtin();
        for file in &builtin.files {
            if !self.directory.join(&file.filename).exists() {
                self.save_file(file)?;
            }
        }
        let mut paths: Vec<_> = fs::read_dir(&self.directory)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            })
            .collect();
        paths.sort_by_key(|path| {
            (
                path.file_name().is_none_or(|name| name != "dynamx.json"),
                path.file_name().unwrap_or_default().to_owned(),
            )
        });
        let mut files = Vec::new();
        let mut warnings = Vec::new();
        for path in paths {
            let result: Result<CatalogFile, String> = (|| {
                if fs::symlink_metadata(&path)
                    .map_err(|e| e.to_string())?
                    .file_type()
                    .is_symlink()
                {
                    return Err("Lien symbolique non pris en charge".into());
                }
                if fs::metadata(&path).map_err(|e| e.to_string())?.len() > 8 * 1024 * 1024 {
                    return Err("Fichier de propriétés trop volumineux".into());
                }
                let mut value: serde_json::Value =
                    serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                if path.file_name().is_some_and(|n| n == "dynamx.json") {
                    if let Some(rules) = value
                        .get_mut("rules")
                        .and_then(serde_json::Value::as_array_mut)
                    {
                        for rule in rules {
                            if let Some(parameters) = rule
                                .get_mut("parameters")
                                .and_then(serde_json::Value::as_array_mut)
                            {
                                for parameter in parameters {
                                    if parameter.get("description").is_none() {
                                        if let Some(name) = parameter
                                            .get("name")
                                            .and_then(serde_json::Value::as_str)
                                        {
                                            let description = builtin_description(name).to_owned();
                                            parameter["description"] = description.into();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                let catalog: CatalogFile =
                    serde_json::from_value(value).map_err(|e| e.to_string())?;
                validate_catalog_file(&catalog)?;
                Ok(catalog)
            })();
            match result {
                Ok(catalog) => files.push(NamedCatalogFile {
                    filename: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    catalog,
                }),
                Err(error) => warnings.push(format!("{} : {error}", path.display())),
            }
        }
        if !files.iter().any(|file| file.filename == "dynamx.json") {
            files.insert(0, builtin.files[0].clone());
        }
        Ok(CatalogLoad {
            catalog: ParameterCatalog { files },
            warnings,
        })
    }
    pub fn save_file(&self, file: &NamedCatalogFile) -> Result<(), String> {
        let filename = &file.filename;
        if !filename.ends_with(".json")
            || filename == ".json"
            || filename.contains(['/', '\\', ':'])
            || filename.starts_with('.')
            || filename.chars().any(char::is_control)
        {
            return Err("Nom de fichier JSON invalide".into());
        }
        validate_catalog_file(&file.catalog)?;
        fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        let mut temp =
            tempfile::NamedTempFile::new_in(&self.directory).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(temp.as_file_mut(), &file.catalog)
            .map_err(|e| e.to_string())?;
        temp.as_file().sync_all().map_err(|e| e.to_string())?;
        temp.persist(self.directory.join(filename))
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub(crate) fn builtin_description(name: &str) -> &'static str {
    match name {
        "Name" => "Nom affiché pour cet objet dans le jeu.",
        "Description" => "Texte de présentation de l’objet ou du pack.",
        "PackName" => "Nom interne du pack, utilisé notamment dans les références aux autres configurations.",
        "PackVersion" => "Version du contenu de ce pack.",
        "DcFileVersion" => "Version du format de configuration DynamX utilisée par ce pack.",
        "CompatibleWithLoaderVersions" => "Versions du chargeur DynamX compatibles avec ce pack.",
        "Id" => "Identifiant de l’addon requis.",
        "Versions" => "Versions acceptées pour cet addon.",
        "Model" => "Chemin du modèle 3D de l’objet, relatif au pack ou sous forme namespace:chemin.",
        "Textures" => "Déclaration des variantes de textures associées au modèle.",
        "EmptyMass" => "Masse à vide de l’objet physique.",
        "DragCoefficient" => "Coefficient de traînée utilisé par la physique du véhicule.",
        "CenterOfGravityOffset" => "Décalage du centre de gravité par rapport au modèle : trois composantes.",
        "ShapeYOffset" => "Décalage vertical des formes de collision.",
        "ScaleModifier" => "Facteurs d’échelle globaux du modèle, sur les trois axes.",
        "Scale" => "Dimensions ou échelle de cette pièce selon son type : trois composantes.",
        "Position" => "Position de la pièce par rapport au modèle. DynamX convertit les axes du format de configuration.",
        "RotationPoint" => "Point autour duquel la pièce effectue sa rotation.",
        "ObjectName" => "Nom de la pièce du modèle OBJ utilisée, par exemple SteeringWheel pour le volant.",
        "Rotation" => "Rotation de base en quaternion DynamX w x y z ; 1 0 0 0 représente l’identité.",
        "SuspensionAxis" => "Orientation de l’axe de suspension sous forme de quaternion w x y z.",
        "DefaultEngine" => "Référence à la configuration du moteur utilisé par défaut (Pack.engine_nom).",
        "DefaultSounds" => "Référence à la configuration des sons utilisés par défaut.",
        "Power" => "Puissance de base du moteur utilisée dans les calculs physiques.",
        "MaxRPM" => "Régime maximal du moteur.",
        "Braking" => "Effet de freinage moteur.",
        "RPMPower" => "Point de la courbe moteur : régime et facteur de puissance.",
        "RPMRange" => "Plage de régime du moteur : valeur minimale et maximale.",
        "SpeedRange" => "Plage de vitesse associée à ce rapport : minimum et maximum.",
        "Sound" => "Ressource sonore à jouer pour cette section.",
        "PitchRange" => "Plage de hauteur du son. Ce paramètre est facultatif.",
        "AttachedWheel" => "Référence à la configuration de roue utilisée par cette pièce (Pack.wheel_nom).",
        "DrivingWheel" => "Indique si cette roue reçoit la puissance du moteur.",
        "IsRight" => "Indique si cette roue est placée à droite du véhicule.",
        "IsSteerable" => "Indique si cette roue peut pivoter pour diriger le véhicule.",
        "MaxTurn" => "Angle maximal de braquage de cette roue.",
        "HandBrakingWheel" => "Indique si le frein à main agit sur cette roue.",
        "DependsOn" => "Pièce dont dépend cette pièce dans la configuration.",
        "MudGuard" => "Nom de la pièce 3D utilisée comme garde-boue.",
        "Radius" => "Rayon de la roue.",
        "RimRadius" => "Rayon de la jante.",
        "Width" => "Largeur de la roue.",
        "Friction" => "Coefficient d’adhérence de la roue.",
        "BrakeForce" => "Force de freinage appliquée à la roue.",
        "HandBrakeForce" => "Force de freinage du frein à main.",
        "RollInInfluence" => "Influence de la roue sur le roulis du véhicule.",
        "SuspensionMaxForce" => "Force maximale de la suspension.",
        "SuspensionRestLength" => "Longueur de la suspension au repos.",
        "SuspensionStiffness" => "Raideur de la suspension.",
        "WheelDampingRelaxation" => "Amortissement de la suspension lorsqu’elle se détend.",
        "WheelsDampingCompression" => "Amortissement de la suspension lorsqu’elle se comprime.",
        "SkidParticle" => "Type de particules produites lors du dérapage.",
        "Driver" => "Indique si ce siège permet de conduire le véhicule.",
        "PlayerPosition" => "Posture du joueur lorsqu’il occupe ce siège.",
        "PlayerSize" => "Dimensions associées au joueur assis sur cette pièce.",
        "LinkedDoorPart" => "Nom de la porte associée à ce siège.",
        "ShouldLimitFieldOfView" => "Active les limites de rotation de la vue pour ce siège.",
        "MinYaw" => "Limite minimale de rotation horizontale de la vue.",
        "MaxYaw" => "Limite maximale de rotation horizontale de la vue.",
        "MinPitch" => "Limite minimale de rotation verticale de la vue.",
        "MaxPitch" => "Limite maximale de rotation verticale de la vue.",
        "CreativeTabName" => "Onglet créatif dans lequel l’objet apparaît.",
        "IconText" => "Texte associé à l’icône de l’objet.",
        "Item3DRenderLocation" => "Contextes dans lesquels l’objet utilise son rendu 3D.",
        "ItemRotate" => "Rotation du modèle lorsqu’il est rendu comme item.",
        "ItemScale" => "Échelle du modèle lorsqu’il est rendu comme item.",
        "ItemTranslate" => "Décalage du modèle lorsqu’il est rendu comme item.",
        "MaxItemStackSize" => "Nombre maximal d’objets dans une pile.",
        "Durability" => "Durabilité de l’item ou de l’armure.",
        "Enchantability" => "Valeur utilisée pour l’enchantement de l’item.",
        "DamageReduction" => "Valeurs de réduction des dégâts de l’armure.",
        "ArmorArms" => "Pièces du modèle associées aux bras de l’armure.",
        "ArmorFoot" => "Pièces du modèle associées aux pieds de l’armure.",
        "ArmorLegs" => "Pièces du modèle associées aux jambes de l’armure.",
        "EquipSound" => "Son joué lorsque l’armure est équipée.",
        "Material" => "Matériau Minecraft utilisé par cet objet.",
        "LightLevel" => "Niveau de lumière émis par le bloc.",
        "UseComplexCollision" => "Active les collisions complexes du modèle.",
        "UseComplexCollisions" => "Active les collisions complexes du modèle.",
        "PlayerStandOnTop" => "Règle indiquant si un joueur peut se tenir sur l’objet.",
        "RenderDistanceSquared" => "Distance de rendu au carré utilisée pour cet objet.",
        "DespawnTime" => "Durée utilisée avant la disparition de cet objet selon son type.",
        "Rotate" => "Rotation du modèle pour ce rendu.",
        "Translate" => "Décalage du modèle pour ce rendu.",
        "AngularDamping" => "Amortissement des mouvements de rotation du véhicule.",
        "LinearDamping" => "Amortissement des déplacements du véhicule.",
        "MaxVehicleSpeed" => "Vitesse maximale du véhicule ; infinite désactive cette limite.",
        "TurnSpeed" => "Vitesse de rotation ou de braquage selon le type de véhicule.",
        "DefaultZoomLevel" => "Niveau de zoom par défaut de la caméra.",
        "SteeringMethod" => "Méthode de direction utilisée par le véhicule.",
        "CameraPositionY" => "Position verticale de la caméra.",
        "CameraRotation" => "Réglage de rotation de la caméra.",
        "Intensity" => "Vecteur d’intensité de la force appliquée par cette pièce.",
        _ => "Paramètre DynamX ou addon. Sa description peut être renseignée dans Propriétés.",
    }
}
