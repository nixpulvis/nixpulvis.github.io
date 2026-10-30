//! The site's data files, in `_data/`, compiled in like the pages.

use serde::Deserialize;

use crate::index::Nav;

/// Everything in `_data/`.
#[derive(Debug)]
pub struct Data {
    /// The header's links, from `nav.yml`.
    pub nav: Vec<Nav>,
    /// The References page's links, from `references.yml`.
    pub references: Vec<Reference>,
    /// How tags are ordered and colored, from `reference_tags.yml`.
    pub tags: Tags,
}

/// A link on the References page.
#[derive(Debug, Clone, Deserialize)]
pub struct Reference {
    pub title: String,
    pub url: String,
    pub tags: Vec<String>,
}

/// Reference tags on a scale, from the deepest computer science to the most
/// general topics, then groups of tags off the scale.
#[derive(Debug, Deserialize)]
pub struct Tags {
    pub scale: Vec<Vec<String>>,
    /// Each group's name, which is its CSS class, and its tags, in the
    /// order they're written.
    pub groups: serde_yaml::Mapping,
}

impl Data {
    /// The data files compiled into the crate.
    pub fn load() -> Result<Self, String> {
        Ok(Self {
            nav: parse("nav.yml", include_str!("../../_data/nav.yml"))?,
            references: parse("references.yml", include_str!("../../_data/references.yml"))?,
            tags: parse(
                "reference_tags.yml",
                include_str!("../../_data/reference_tags.yml"),
            )?,
        })
    }

    /// The references by title, ignoring case.
    pub fn references_by_title(&self) -> Vec<&Reference> {
        let mut references: Vec<&Reference> = self.references.iter().collect();
        references.sort_by_key(|r| r.title.to_lowercase());
        references
    }

    /// Every tag used, sorted.
    pub fn used_tags(&self) -> Vec<&str> {
        let mut tags: Vec<&str> = self
            .references
            .iter()
            .flat_map(|r| r.tags.iter().map(String::as_str))
            .collect();
        tags.sort_unstable();
        tags.dedup();
        tags
    }

    /// Every tag used, in scale order, then the groups, then the rest.
    pub fn ordered_tags(&self) -> Vec<&str> {
        let used = self.used_tags();
        let mut ordered: Vec<&str> = self
            .tags
            .steps()
            .flatten()
            .filter(|tag| used.contains(tag))
            .collect();
        for tag in used {
            if !ordered.contains(&tag) {
                ordered.push(tag);
            }
        }
        ordered
    }

    /// How many references have `tag`.
    pub fn tag_count(&self, tag: &str) -> usize {
        self.references
            .iter()
            .filter(|r| r.tags.iter().any(|t| t == tag))
            .count()
    }
}

fn parse<T: serde::de::DeserializeOwned>(name: &str, yaml: &str) -> Result<T, String> {
    serde_yaml::from_str(yaml).map_err(|e| format!("_data/{name}: {e}"))
}

impl Tags {
    /// The CSS class coloring `tag`: `scale-<n>` for the nth step of the
    /// scale, a group's name, or nothing.
    pub fn class(&self, tag: &str) -> String {
        let mut class = String::new();
        for (i, step) in self.scale.iter().enumerate() {
            if step.iter().any(|t| t == tag) {
                class = format!("scale-{}", i + 1);
            }
        }
        for (name, tags) in self.groups() {
            if tags.contains(&tag) {
                class = name.to_owned();
            }
        }
        class
    }

    /// The scale's steps, then each group, as lists of tags.
    fn steps(&self) -> impl Iterator<Item = Vec<&str>> {
        self.scale
            .iter()
            .map(|step| step.iter().map(String::as_str).collect())
            .chain(self.groups().map(|(_, tags)| tags))
    }

    fn groups(&self) -> impl Iterator<Item = (&str, Vec<&str>)> {
        self.groups.iter().filter_map(|(name, tags)| {
            let tags = tags
                .as_sequence()?
                .iter()
                .filter_map(|t| t.as_str())
                .collect();
            Some((name.as_str()?, tags))
        })
    }
}
