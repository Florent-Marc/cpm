use std::fmt;
use std::io::{self, Read, Write};
use std::ops::Range;

/// Erreur syntaxique avec numéro de ligne (à partir de 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: &'static str,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ligne {} : {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Syntax(ParseError),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Syntax(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Syntax(error) => Some(error),
        }
    }
}

/// Section identifiée par sa position dans `Document::sections()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    pub parent: Option<usize>,
    pub line: usize,
    closing_offset: usize,
}

/// Propriété identifiée par sa position dans `Document::properties()`.
/// `section == None` indique une propriété à la racine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub key: String,
    pub value: String,
    pub section: Option<usize>,
    pub line: usize,
    value_range: Range<usize>,
    line_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    UnknownProperty(usize),
    MultilineValue,
    InvalidKey,
    UnknownSection(usize),
    DuplicateProperty,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProperty(index) => write!(f, "propriété inconnue : {index}"),
            Self::MultilineValue => write!(f, "une valeur doit tenir sur une seule ligne"),
            Self::InvalidKey => write!(f, "nom de paramètre invalide"),
            Self::UnknownSection(index) => write!(f, "section inconnue : {index}"),
            Self::DuplicateProperty => write!(f, "ce paramètre existe déjà dans cette section"),
        }
    }
}

impl std::error::Error for EditError {}

/// Document conservant exactement sa source, sans normaliser les commentaires,
/// l'ordre, les doublons, les espaces, le BOM ou les fins de ligne.
#[derive(Debug, Clone)]
pub struct Document {
    source: String,
    properties: Vec<Property>,
    sections: Vec<Section>,
}

impl Document {
    /// Analyse la syntaxe ligne par ligne ; ne valide pas les règles métier.
    pub fn parse(source: &str) -> Result<Self, ParseError> {
        let mut document = Self {
            source: source.to_owned(),
            properties: Vec::new(),
            sections: Vec::new(),
        };
        let mut stack: Vec<usize> = Vec::new();
        let mut comment_start = None;
        let mut offset = 0;
        for (index, raw) in source.split_inclusive('\n').enumerate() {
            let line = index + 1;
            let body = raw.trim_end_matches(['\r', '\n']);
            let body = if index == 0 {
                body.strip_prefix('\u{feff}').unwrap_or(body)
            } else {
                body
            };
            let text = body.trim();
            let error = |message| ParseError { line, message };
            if comment_start.is_some() {
                if text.ends_with("*/") {
                    comment_start = None;
                }
            } else if text.starts_with("/*") {
                if !text.ends_with("*/") {
                    comment_start = Some(line);
                }
            } else if text.is_empty() || text.starts_with("//") {
                // La source conserve les lignes sans contenu sémantique.
            } else if text == "}" {
                if let Some(id) = stack.pop() {
                    document.sections[id].closing_offset = offset;
                } else {
                    return Err(error("accolade fermante sans section ouverte"));
                }
            } else if let Some((key, value)) = text.split_once(':') {
                let key = key.trim();
                if key.is_empty() || key.contains(['{', '}']) {
                    return Err(error("nom de propriété invalide"));
                }
                // Chercher le premier ':' dans raw préserve les offsets UTF-8/BOM.
                let colon = raw.find(':').expect("propriété contenant ':'");
                let after = &raw.trim_end_matches(['\r', '\n'])[colon + 1..];
                let start = colon + 1 + after.len() - after.trim_start().len();
                let end = start + after.trim().len();
                document.properties.push(Property {
                    key: key.to_owned(),
                    value: value.trim().to_owned(),
                    section: stack.last().copied(),
                    line,
                    value_range: offset + start..offset + end,
                    line_range: offset
                        + if index == 0 && raw.starts_with('\u{feff}') {
                            '\u{feff}'.len_utf8()
                        } else {
                            0
                        }..offset + raw.len(),
                });
            } else if let Some(name) = text.strip_suffix('{') {
                if name.is_empty()
                    || name.ends_with(char::is_whitespace)
                    || name.contains(['{', '}'])
                {
                    return Err(error("nom de section invalide ; '{' doit suivre le nom"));
                }
                let id = document.sections.len();
                document.sections.push(Section {
                    name: name.to_owned(),
                    parent: stack.last().copied(),
                    line,
                    closing_offset: 0,
                });
                stack.push(id);
            } else {
                return Err(error("propriété ou section attendue"));
            }
            offset += raw.len();
        }
        if let Some(line) = comment_start {
            return Err(ParseError {
                line,
                message: "commentaire non terminé",
            });
        }
        if let Some(id) = stack.last() {
            return Err(ParseError {
                line: document.sections[*id].line,
                message: "section non terminée",
            });
        }
        Ok(document)
    }

    /// Lit un flux UTF-8. Les erreurs d'encodage sont des erreurs d'entrée/sortie.
    pub fn read_from(mut reader: impl Read) -> Result<Self, ReadError> {
        let mut source = String::new();
        reader.read_to_string(&mut source).map_err(ReadError::Io)?;
        Self::parse(&source).map_err(ReadError::Syntax)
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn properties(&self) -> &[Property] {
        &self.properties
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Écrit le texte courant. L'appelant gère la sauvegarde atomique sur disque.
    pub fn write_to(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_all(self.source.as_bytes())
    }

    /// Supprime uniquement la ligne du paramètre, en conservant commentaires,
    /// sections et BOM. Les indices des propriétés sont recalculés.
    pub fn remove_property(&mut self, index: usize) -> Result<(), EditError> {
        let property = self
            .properties
            .get(index)
            .ok_or(EditError::UnknownProperty(index))?;
        let mut source = self.source.clone();
        source.replace_range(property.line_range.clone(), "");
        *self = Self::parse(&source).expect("suppression d'une ligne de propriété");
        Ok(())
    }

    /// Ajoute un paramètre sans reconstruire le document ni modifier les autres
    /// lignes. Les indices de propriétés peuvent changer après l'insertion.
    pub fn add_property(
        &mut self,
        section: Option<usize>,
        key: &str,
        value: &str,
    ) -> Result<(), EditError> {
        if key.is_empty()
            || !key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c))
        {
            return Err(EditError::InvalidKey);
        }
        if value.contains(['\r', '\n']) {
            return Err(EditError::MultilineValue);
        }
        if self
            .properties
            .iter()
            .any(|p| p.section == section && p.key == key)
        {
            return Err(EditError::DuplicateProperty);
        }
        let newline = if self.source.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let mut source = self.source.clone();
        if let Some(id) = section {
            let section = self.sections.get(id).ok_or(EditError::UnknownSection(id))?;
            let closing = &self.source[section.closing_offset..];
            let indent: String = closing
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .collect();
            source.insert_str(
                section.closing_offset,
                &format!("{indent}    {key}: {}{newline}", value.trim()),
            );
        } else {
            if !source.is_empty() && !source.ends_with('\n') {
                source.push_str(newline);
            }
            source.push_str(&format!("{key}: {}{newline}", value.trim()));
        }
        *self = Self::parse(&source).expect("insertion contrôlée d'un paramètre");
        Ok(())
    }

    /// Remplace seulement la valeur indiquée, même si une clé apparaît plusieurs
    /// fois. Les indices restent stables ; les espaces externes sont conservés.
    pub fn set_value(&mut self, index: usize, value: &str) -> Result<(), EditError> {
        if value.contains(['\r', '\n']) {
            return Err(EditError::MultilineValue);
        }
        let property = self
            .properties
            .get(index)
            .ok_or(EditError::UnknownProperty(index))?;
        let range = property.value_range.clone();
        let value = value.trim();
        let mut source = self.source.clone();
        source.replace_range(range, value);
        // Une édition ne peut changer ni les clés ni la structure du document.
        *self = Self::parse(&source).expect("édition d'une valeur sur une seule ligne");
        Ok(())
    }
}
