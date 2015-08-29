
extern crate time;

use propnode_parser as propnode_parser;

use std::collections::HashMap;
use std::mem;
use std::cell::RefCell;
use std::rc::Rc;

use document::*;
use propnode::*;
use interface::*;

pub struct System {
    document: Document,
    prev_frame_time: time::Timespec,
    sub_systems: Vec<Rc<RefCell<Box<ISubSystem>>>>,
    invalidated_properties: Vec<PropRef>,
    pub running: bool
}

impl System {
    pub fn new() -> System {
        let mut pyramid = System {
            document: Document::new(),
            prev_frame_time: time::get_time(),
            sub_systems: vec![],
            invalidated_properties: vec![],
            running: true
        };
        return pyramid;
    }
    pub fn add_subsystem(&mut self, subsystem: Box<ISubSystem>) {
        self.sub_systems.push(Rc::new(RefCell::new(subsystem)));
    }
    pub fn set_document(&mut self, document: Document) {
        self.document = document;
        for system in self.sub_systems.clone() {
            system.borrow_mut().on_document_loaded(self);
        }
    }
    pub fn update(&mut self) {
        let t = time::get_time();
        let diff_time = t - self.prev_frame_time;
        for system in self.sub_systems.clone() {
            system.borrow_mut().update(self, diff_time);
        }
        while {
            let ips = mem::replace(&mut self.invalidated_properties, vec![]);
            self.on_property_value_change(&ips);
            self.invalidated_properties.len() > 0
        } {};
        self.prev_frame_time = t;
    }
    fn on_entity_added(&mut self, entity_id: &EntityId) {
        for system in self.sub_systems.clone() {
            system.borrow_mut().on_entity_added(self, entity_id);
        }
    }
    fn on_property_value_change(&mut self, prop_refs: &Vec<PropRef>) {
        for system in self.sub_systems.clone() {
            system.borrow_mut().on_property_value_change(self, prop_refs);
        }
    }
}

impl ISystem for System {
    fn append_entity(&mut self, parent: &EntityId, type_name: String, name: Option<String>) -> Result<EntityId, DocError> {
        match self.document.append_entity(parent.clone(), type_name, name) {
            Ok(entity_id) => {
                self.on_entity_added(&entity_id);
                Ok(entity_id)
            },
            err @ _ => err
        }
    }
    fn get_entity_by_name(&self, name: &str) -> Option<EntityId> {
        self.document.get_entity_by_name(name)
    }
    fn set_property(&mut self, entity_id: &EntityId, name: String, value: PropNode) {
        let invalid_props = self.document.set_property(entity_id, &name.as_str(), value).ok().unwrap();
        self.invalidated_properties.push_all(&invalid_props);
    }
    fn get_property_value(&self, entity_id: &EntityId, name: &str) -> Result<PropNode, DocError> {
        self.document.get_property_value(entity_id, name)
    }
    fn has_property(&self, entity_id: &EntityId, name: &str) -> Result<bool, DocError> {
        self.document.has_property(entity_id, name)
    }
    fn resolve_named_prop_ref(&self, entity_id: &EntityId, named_prop_ref: &NamedPropRef) -> Result<PropRef, DocError> {
        self.document.resolve_named_prop_ref(entity_id, named_prop_ref)
    }
    fn get_entity_type_name(&self, entity_id: &EntityId) -> Result<&String, DocError> {
        self.document.get_entity_type_name(entity_id)
    }
    fn get_properties(&self, entity_id: &EntityId) -> Result<Vec<PropRef>, DocError> {
        self.document.get_properties(entity_id)
    }
    fn get_children(&self, entity_id: &EntityId) -> Result<&Vec<EntityId>, DocError> {
        self.document.get_children(entity_id)
    }
    fn get_entities(&self) -> EntityIter {
        self.document.iter()
    }
    fn get_root(&self) -> &EntityId {
        self.document.get_root()
    }
    fn exit(&mut self) {
        self.running = false;
    }
}
