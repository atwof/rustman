use serde::{Deserialize, Serialize};

/// A folder node inside a collection. Groups nest arbitrarily deep, which is
/// what gives the sidebar its `Collection → Backend → Resource → Endpoint`
/// hierarchy (issue #53): a "backend" is a group directly under a collection,
/// a "resource" is a group under a backend, and an endpoint is a request leaf.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    /// The enclosing collection's id, or the parent group's id when nested.
    pub parent_id: String,
    /// Discriminator for `parent_id`: collection ids and group ids are both
    /// UUIDs, so the kind has to be stored alongside.
    pub parent_is_collection: bool,
    pub name: String,
    /// Manual order among the siblings sharing the same parent. Lower sorts
    /// first; stable for items that were never reordered.
    pub sort_order: i64,
}

impl Group {
    pub fn new(parent_id: String, parent_is_collection: bool, name: String, sort_order: i64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id,
            parent_is_collection,
            name,
            sort_order,
        }
    }
}

/// The tree as reconstructed from the DB: a collection's direct children,
/// with groups carrying their own nested children. Used by the importers and
/// by git sync so a repository's folder structure survives a round trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedGroup {
    pub id: String,
    pub parent_id: String,
    pub parent_is_collection: bool,
    pub name: String,
    pub sort_order: i64,
    pub requests: Vec<crate::domain::collection::SavedRequest>,
    pub groups: Vec<ImportedGroup>,
}

/// One imported collection plus the full tree inside it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedCollection {
    pub collection: crate::domain::collection::Collection,
    pub requests: Vec<crate::domain::collection::SavedRequest>,
    pub groups: Vec<ImportedGroup>,
}

impl ImportedCollection {
    /// Every group in this collection, depth-first (parents before children).
    pub fn all_groups(&self) -> Vec<&ImportedGroup> {
        fn walk<'a>(g: &'a ImportedGroup, out: &mut Vec<&'a ImportedGroup>) {
            out.push(g);
            for child in &g.groups {
                walk(child, out);
            }
        }
        let mut out = Vec::new();
        for g in &self.groups {
            walk(g, &mut out);
        }
        out
    }

    /// Every request in this collection, depth-first.
    pub fn all_requests(&self) -> Vec<&crate::domain::collection::SavedRequest> {
        fn walk<'a>(g: &'a ImportedGroup, out: &mut Vec<&'a crate::domain::collection::SavedRequest>) {
            out.extend(g.requests.iter());
            for child in &g.groups {
                walk(child, out);
            }
        }
        let mut out: Vec<&crate::domain::collection::SavedRequest> = self.requests.iter().collect();
        for g in &self.groups {
            walk(g, &mut out);
        }
        out
    }
}
