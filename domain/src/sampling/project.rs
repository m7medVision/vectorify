//! The `Project` aggregate root — the single consistency boundary for layers and
//! features. All mutations go through its methods, which enforce invariants and
//! return the domain events they emitted.

use serde::{Deserialize, Serialize};

use super::feature::{Category, Feature, FeatureId, Kind};
use super::layer::{LayerRef, XyzTemplate};
use crate::events::DomainEvent;

#[derive(Default, Serialize, Deserialize)]
pub struct Project {
    layers: Vec<LayerRef>,
    features: Vec<Feature>,
    next_id: u64,
}

impl Project {
    pub fn new() -> Self {
        Self { layers: vec![], features: vec![], next_id: 1 }
    }

    // ---- read access ----
    pub fn layers(&self) -> &[LayerRef] {
        &self.layers
    }
    pub fn features(&self) -> &[Feature] {
        &self.features
    }
    pub fn feature(&self, id: FeatureId) -> Option<&Feature> {
        self.features.iter().find(|f| f.id == id)
    }

    // ---- layer operations ----
    pub fn add_raster_layer(&mut self, path: impl Into<String>) -> Vec<DomainEvent> {
        self.layers.push(LayerRef::Raster { path: path.into() });
        vec![DomainEvent::LayerAdded]
    }

    pub fn add_basemap_layer(&mut self, template: XyzTemplate) -> Vec<DomainEvent> {
        self.layers.push(LayerRef::Basemap { url: template.as_str().to_string() });
        vec![DomainEvent::LayerAdded]
    }

    pub fn remove_layer(&mut self, index: usize) -> Result<Vec<DomainEvent>, String> {
        if index >= self.layers.len() {
            return Err("layer index out of range".into());
        }
        self.layers.remove(index);
        Ok(vec![DomainEvent::LayerRemoved])
    }

    /// Swap a layer with its neighbour (delta -1 = up, +1 = down). Out-of-range is a no-op.
    pub fn reorder_layer(&mut self, index: usize, delta: isize) {
        let j = index as isize + delta;
        if index < self.layers.len() && j >= 0 && (j as usize) < self.layers.len() {
            self.layers.swap(index, j as usize);
        }
    }

    // ---- feature operations ----
    pub fn add_polygon(
        &mut self,
        coords: Vec<[f64; 2]>,
        category: Category,
    ) -> Result<(FeatureId, Vec<DomainEvent>), String> {
        Feature::geometry_ok(Kind::Polygon, &coords)?;
        Ok(self.insert(Kind::Polygon, coords, category))
    }

    pub fn add_point(
        &mut self,
        coord: [f64; 2],
        category: Category,
    ) -> Result<(FeatureId, Vec<DomainEvent>), String> {
        let coords = vec![coord];
        Feature::geometry_ok(Kind::Point, &coords)?;
        Ok(self.insert(Kind::Point, coords, category))
    }

    fn insert(
        &mut self,
        kind: Kind,
        coords: Vec<[f64; 2]>,
        category: Category,
    ) -> (FeatureId, Vec<DomainEvent>) {
        let id = self.mint_id();
        self.features.push(Feature { id, kind, coords, category });
        (id, vec![DomainEvent::FeatureDrawn(id)])
    }

    pub fn move_vertex(
        &mut self,
        id: FeatureId,
        vertex: usize,
        coord: [f64; 2],
    ) -> Result<Vec<DomainEvent>, String> {
        let f = self.features.iter_mut().find(|f| f.id == id).ok_or("no such feature")?;
        let v = f.coords.get_mut(vertex).ok_or("vertex out of range")?;
        *v = coord;
        Ok(vec![DomainEvent::FeatureMoved(id)])
    }

    /// Replace a polygon's entire ring atomically, re-validating the geometry
    /// invariant. Backs OpenLayers `Modify` (vertex move/insert/delete), which the
    /// index-based `move_vertex` cannot express.
    pub fn reshape_polygon(
        &mut self,
        id: FeatureId,
        coords: Vec<[f64; 2]>,
    ) -> Result<Vec<DomainEvent>, String> {
        let f = self.features.iter_mut().find(|f| f.id == id).ok_or("no such feature")?;
        if f.kind != Kind::Polygon {
            return Err("not a polygon".into());
        }
        Feature::geometry_ok(Kind::Polygon, &coords)?;
        f.coords = coords;
        Ok(vec![DomainEvent::FeatureMoved(id)])
    }

    pub fn delete_feature(&mut self, id: FeatureId) -> Result<Vec<DomainEvent>, String> {
        let before = self.features.len();
        self.features.retain(|f| f.id != id);
        if self.features.len() == before {
            return Err("no such feature".into());
        }
        Ok(vec![DomainEvent::FeatureDeleted(id)])
    }

    pub fn set_category(
        &mut self,
        id: FeatureId,
        category: Category,
    ) -> Result<Vec<DomainEvent>, String> {
        let f = self.features.iter_mut().find(|f| f.id == id).ok_or("no such feature")?;
        f.category = category;
        Ok(vec![DomainEvent::FeatureCategorized(id)])
    }

    fn mint_id(&mut self) -> FeatureId {
        let id = FeatureId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Re-check invariants after deserialization (loading a project file bypasses
    /// the constructors). Repairs `next_id` so future ids stay unique.
    pub fn validate(&mut self) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        let mut max_id = 0;
        for f in &self.features {
            if !seen.insert(f.id) {
                return Err(format!("duplicate feature id {:?}", f.id));
            }
            Feature::geometry_ok(f.kind, &f.coords)?;
            max_id = max_id.max(f.id.0);
        }
        if self.next_id <= max_id {
            self.next_id = max_id + 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat() -> Category {
        Category::new("sample").unwrap()
    }

    #[test]
    fn rejects_short_polygon() {
        let mut p = Project::new();
        assert!(p.add_polygon(vec![[0.0, 0.0], [1.0, 1.0]], cat()).is_err());
        assert_eq!(p.features().len(), 0);
    }

    #[test]
    fn reshape_polygon_replaces_ring() {
        let mut p = Project::new();
        let (id, _) = p.add_polygon(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], cat()).unwrap();
        p.reshape_polygon(id, vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]).unwrap();
        assert_eq!(p.feature(id).unwrap().coords.len(), 4);
    }

    #[test]
    fn reshape_polygon_rejects_short_ring_and_keeps_original() {
        let mut p = Project::new();
        let orig = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let (id, _) = p.add_polygon(orig.clone(), cat()).unwrap();
        assert!(p.reshape_polygon(id, vec![[0.0, 0.0], [1.0, 1.0]]).is_err());
        assert_eq!(p.feature(id).unwrap().coords, orig);
    }

    #[test]
    fn reshape_rejects_non_polygon() {
        let mut p = Project::new();
        let (id, _) = p.add_point([0.0, 0.0], cat()).unwrap();
        assert!(p.reshape_polygon(id, vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]).is_err());
    }

    #[test]
    fn feature_ids_are_unique() {
        let mut p = Project::new();
        let (a, _) = p.add_point([0.0, 0.0], cat()).unwrap();
        let (b, _) = p.add_point([1.0, 1.0], cat()).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn drawing_emits_event() {
        let mut p = Project::new();
        let (id, events) = p.add_point([0.0, 0.0], cat()).unwrap();
        assert_eq!(events, vec![DomainEvent::FeatureDrawn(id)]);
    }

    #[test]
    fn validate_repairs_next_id() {
        let mut p = Project::new();
        p.features.push(Feature {
            id: FeatureId(42),
            kind: Kind::Point,
            coords: vec![[0.0, 0.0]],
            category: cat(),
        });
        p.validate().unwrap();
        let (id, _) = p.add_point([1.0, 1.0], cat()).unwrap();
        assert_eq!(id, FeatureId(43));
    }
}
