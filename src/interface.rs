
use pon::*;
use document::*;
use system::*;

pub trait ISubSystem {
    fn on_document_loaded(&mut self, system: &mut System) {
        let entities: Vec<EntityId> = { system.document().entities_iter().map(|x| x.clone()).collect() };
        for entity_id in entities {
            self.on_entity_added(system, &entity_id);
        }
    }
    fn on_entity_added(&mut self, system: &mut System, entity_id: &EntityId) {
        let prop_refs: Vec<PropRef> = { system.document().get_properties(&entity_id).unwrap() };
        self.on_property_value_change(system, &prop_refs);
    }
    fn on_property_value_change(&mut self, system: &mut System, prop_refs: &Vec<PropRef>) {}
    fn update(&mut self, system: &mut System) {}
}
