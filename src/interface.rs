
use time;

use propnode::*;
use document::*;

pub trait System {
    fn append_entity(&mut self, parent: &EntityId, type_name: String, name: Option<String>) -> Result<EntityId, DocError>;
    fn get_entity_by_name(&self, name: &str) -> Option<EntityId>;
    fn set_property(&mut self, entity_id: &EntityId, name: String, value: PropNode);
    fn get_property_value(&self, entity_id: &EntityId, name: &str) -> Result<PropNode, DocError>;
    fn resolve_named_prop_ref(&self, entity_id: &EntityId, named_prop_ref: &NamedPropRef) -> Result<PropRef, DocError>;
    fn get_properties(&self, entity_id: &EntityId) -> Result<Vec<PropRef>, DocError>;
    fn get_children(&self, entity_id: &EntityId) -> Result<&Vec<EntityId>, DocError>;
    fn exit(&mut self);
}

pub trait SubSystem {
    fn on_entity_added(&mut self, system: &mut System, entity_id: &EntityId);
    fn on_property_value_change(&mut self, system: &mut System, prop_refs: &Vec<PropRef>);
    fn update(&mut self, system: &mut System, delta_time: time::Duration);
}
