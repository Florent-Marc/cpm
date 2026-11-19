//! Catalogue initial basé sur le wiki DynamX. Les paramètres d'addons inconnus
//! sont conservés et peuvent être ajoutés sans être déclarés obligatoires.
use crate::Document;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct ParameterSpec {
    pub key: &'static str,
    pub default_value: &'static str,
    pub required: bool,
    pub allow_empty: bool,
    pub aliases: &'static [&'static str],
}
const fn r(key: &'static str) -> ParameterSpec {
    ParameterSpec {
        key,
        default_value: "",
        required: true,
        allow_empty: false,
        aliases: &[],
    }
}
const fn o(key: &'static str, value: &'static str) -> ParameterSpec {
    ParameterSpec {
        key,
        default_value: value,
        required: false,
        allow_empty: true,
        aliases: &[],
    }
}
fn base_item() -> Vec<ParameterSpec> {
    vec![
        r("Name"),
        ParameterSpec {
            allow_empty: true,
            ..r("Description")
        },
        r("Model"),
        ParameterSpec {
            aliases: &["CreativeTab", "TabName"],
            ..o("CreativeTabName", "")
        },
        o("IconText", ""),
        o("Item3DRenderLocation", "all"),
        o("ItemRotate", "0 0 0"),
        o("ItemScale", "0.9"),
        o("ItemTranslate", "0 0 0"),
    ]
}

pub fn parameter_catalog(
    path: &Path,
    document: &Document,
    section: Option<usize>,
) -> Vec<ParameterSpec> {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let kind = name.split('_').next().unwrap_or("");
    if let Some(index) = section {
        let Some(section) = document.sections().get(index) else {
            return Vec::new();
        };
        let s = section.name.as_str();
        if name == "pack_info.dynx" && s == "RequiredAddon" {
            return vec![r("Id"), o("Versions", "")];
        }
        if kind == "engine" {
            if s.starts_with("Point_") {
                return vec![r("RPMPower")];
            }
            if s.starts_with("Gear_") {
                return vec![r("SpeedRange"), r("RPMRange")];
            }
        }
        if ["vehicle", "trailer", "boat", "helicopter"].contains(&kind) {
            if s.starts_with("Shape") {
                return vec![r("Position"), r("Scale"), o("Type", "BOX")];
            }
            if s.starts_with("SteeringWheel") {
                return vec![
                    o("ObjectName", "SteeringWheel"),
                    o("Position", ""),
                    o("Rotation", "1 0 0 0"),
                ];
            }
            if s.starts_with("ForcePoint") {
                return vec![r("Position"), r("Intensity")];
            }
            if s.starts_with("Wheel") {
                return vec![
                    r("AttachedWheel"),
                    r("DrivingWheel"),
                    ParameterSpec {
                        aliases: &["isRight"],
                        ..r("IsRight")
                    },
                    r("IsSteerable"),
                    r("MaxTurn"),
                    r("Position"),
                    o("HandBrakingWheel", "false"),
                    o("DependsOn", ""),
                    o("MudGuard", ""),
                    o("RotationPoint", "0 0 0"),
                    o("Scale", "1 1 1"),
                    o("SuspensionAxis", "1 0 0 0"),
                ];
            }
            if s.starts_with("Seat") || s.ends_with("Seat") {
                return vec![
                    r("Driver"),
                    r("Position"),
                    o("CameraPositionY", "0"),
                    o("CameraRotation", "0"),
                    o("DependsOn", ""),
                    o("LinkedDoorPart", ""),
                    o("MaxPitch", "-105"),
                    o("MaxYaw", "-105"),
                    o("MinPitch", "105"),
                    o("MinYaw", "105"),
                    o("PlayerPosition", "SITTING"),
                    o("PlayerSize", "1 1 1"),
                    o("Rotation", "1 0 0 0"),
                    o("Scale", "1 1 1"),
                    o("ShouldLimitFieldOfView", "true"),
                ];
            }
        }
        if ["sound", "sounds"].contains(&kind)
            && section.parent.is_some_and(|parent| {
                ["Interior", "Exterior"].contains(&document.sections()[parent].name.as_str())
            })
        {
            return vec![r("Sound"), o("PitchRange", "")];
        }
        return Vec::new();
    }
    if name == "pack_info.dynx" {
        return vec![
            r("PackName"),
            ParameterSpec {
                default_value: "12.5.0",
                ..r("DcFileVersion")
            },
            o("PackVersion", "1.0.0"),
            o("CompatibleWithLoaderVersions", "[1.1.0,)"),
            o("Description", ""),
        ];
    }
    match kind {
        "engine" => vec![
            r("Power"),
            r("MaxRPM"),
            r("Braking"),
            o("SteeringMethod", "0"),
            o("TurnSpeed", "0.09"),
        ],
        "wheel" => vec![
            r("BrakeForce"),
            r("Friction"),
            r("Model"),
            r("Radius"),
            r("RimRadius"),
            r("RollInInfluence"),
            r("SuspensionMaxForce"),
            r("SuspensionRestLength"),
            r("SuspensionStiffness"),
            r("WheelDampingRelaxation"),
            r("WheelsDampingCompression"),
            r("Width"),
            o("HandBrakeForce", ""),
            o("ScaleModifier", "1 1 1"),
            o("SkidParticle", "smoke"),
            o("Textures", ""),
        ],
        "vehicle" | "trailer" | "boat" | "helicopter" => {
            let mut specs = base_item();
            specs.extend([
                r("CenterOfGravityOffset"),
                r("DragCoefficient"),
                r("EmptyMass"),
                o("AngularDamping", "0.9"),
                o("LinearDamping", "0.5"),
                o("DefaultEngine", ""),
                o("DefaultSounds", ""),
                o("DefaultZoomLevel", "4"),
                o("MaxVehicleSpeed", ""),
                o("PlayerStandOnTop", "ALWAYS"),
                o("ScaleModifier", "1 1 1"),
                o("ShapeYOffset", ""),
                o("UseComplexCollisions", "true"),
                o("Textures", ""),
            ]);
            specs
        }
        "item" | "armor" | "block" | "prop" => {
            let mut specs = base_item();
            match kind {
                "item" => specs.push(o("MaxItemStackSize", "1")),
                "armor" => specs.extend([
                    o("ArmorArms", ""),
                    o("ArmorBody", ""),
                    o("ArmorFoot", ""),
                    o("ArmorHead", ""),
                    o("ArmorLegs", ""),
                    o("DamageReduction", "1 2 3 1"),
                    o("Durability", "5"),
                    o("Enchantability", "15"),
                    o("EquipSound", "item.armor.equip_leather"),
                ]),
                _ => specs.extend([
                    o("LightLevel", "0"),
                    o("Material", "ROCK"),
                    o("RenderDistanceSquared", "4096"),
                    o("Rotate", "0 0 0"),
                    o("Scale", "1 1 1"),
                    o("Translate", "0 0 0"),
                    o("UseComplexCollision", "true"),
                    o("EmptyMass", ""),
                    o("CenterOfGravityOffset", ""),
                    o("DespawnTime", ""),
                ]),
            }
            specs
        }
        _ => Vec::new(),
    }
}

pub fn has_parameter(document: &Document, section: Option<usize>, spec: &ParameterSpec) -> bool {
    document.properties().iter().any(|p| {
        p.section == section && (p.key == spec.key || spec.aliases.contains(&p.key.as_str()))
    })
}

/// Absence, valeur obligatoire vide ou doublon ambigu. Ne valide pas encore
/// les plages numériques, les références de ressources ou les schémas d'addons.
pub fn configuration_issues(path: &Path, document: &Document) -> Vec<String> {
    let mut issues = Vec::new();
    for section in std::iter::once(None).chain((0..document.sections().len()).map(Some)) {
        for spec in parameter_catalog(path, document, section)
            .into_iter()
            .filter(|spec| spec.required)
        {
            let values: Vec<_> = document
                .properties()
                .iter()
                .filter(|p| {
                    p.section == section
                        && (p.key == spec.key || spec.aliases.contains(&p.key.as_str()))
                })
                .collect();
            let problem = if values.is_empty() {
                Some("paramètre obligatoire absent")
            } else if values.len() > 1 {
                Some("paramètre obligatoire dupliqué")
            } else if !spec.allow_empty && values[0].value.trim().is_empty() {
                Some("valeur obligatoire vide")
            } else {
                None
            };
            if let Some(problem) = problem {
                let context = section
                    .map(|id| document.sections()[id].name.as_str())
                    .unwrap_or("Racine");
                issues.push(format!("{context} / {} : {problem}", spec.key));
            }
        }
    }
    issues
}
