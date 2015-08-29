
use time;

use propnode::*;
use document::*;

pub trait ISystem {
    fn append_entity(&mut self, parent: &EntityId, type_name: String, name: Option<String>) -> Result<EntityId, DocError>;
    fn get_entity_by_name(&self, name: &str) -> Option<EntityId>;
    fn set_property(&mut self, entity_id: &EntityId, name: String, value: PropNode);
    fn get_property_value(&self, entity_id: &EntityId, name: &str) -> Result<PropNode, DocError>;
    fn has_property(&self, entity_id: &EntityId, name: &str) -> Result<bool, DocError>;
    fn resolve_named_prop_ref(&self, entity_id: &EntityId, named_prop_ref: &NamedPropRef) -> Result<PropRef, DocError>;
    fn get_entity_type_name(&self, entity_id: &EntityId) -> Result<&String, DocError>;
    fn get_properties(&self, entity_id: &EntityId) -> Result<Vec<PropRef>, DocError>;
    fn get_children(&self, entity_id: &EntityId) -> Result<&Vec<EntityId>, DocError>;
    fn get_entities(&self) -> EntityIter;
    fn get_root(&self) -> &EntityId;
    fn exit(&mut self);
}

pub trait ISubSystem {
    fn on_document_loaded(&mut self, system: &mut ISystem) {
        let entities: Vec<EntityId> = { system.get_entities().map(|x| x.clone()).collect() };
        for entity_id in entities {
            self.on_entity_added(system, &entity_id);
        }
    }
    fn on_entity_added(&mut self, system: &mut ISystem, entity_id: &EntityId) {}
    fn on_property_value_change(&mut self, system: &mut ISystem, prop_refs: &Vec<PropRef>) {}
    fn update(&mut self, system: &mut ISystem, delta_time: time::Duration) {}
}
