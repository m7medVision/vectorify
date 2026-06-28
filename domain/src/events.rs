//! Domain events. Aggregate methods return these (no global bus, no &mut sink) so
//! the domain stays pure; the application forwards them to subscribers.

use crate::sampling::FeatureId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainEvent {
    LayerAdded,
    LayerRemoved,
    FeatureDrawn(FeatureId),
    FeatureMoved(FeatureId),
    FeatureDeleted(FeatureId),
    FeatureCategorized(FeatureId),
    ProjectExported,
    ProjectSaved,
    ProjectOpened,
}
