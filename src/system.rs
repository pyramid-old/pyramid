
extern crate time;

use std::mem;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;

use document::*;
use pon::*;
use interface::*;

pub struct System {
    pub document: Document,
    prev_frame_time: time::Timespec,
    sub_systems: Vec<Rc<RefCell<Box<ISubSystem>>>>,
    changed_properties: HashSet<PropRef>,
    added_entities: Vec<EntityId>,
    pub running: bool
}

impl System {
    pub fn new() -> System {
        let pyramid = System {
            document: Document::new(),
            prev_frame_time: time::get_time(),
            sub_systems: vec![],
            changed_properties: HashSet::new(),
            added_entities: vec![],
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
    fn build_property_cascades(&mut self) -> Vec<PropRef> {
        let mut ips = mem::replace(&mut self.changed_properties, HashSet::new());
        let mut queue: Vec<PropRef> = ips.clone().into_iter().collect();
        while queue.len() > 0 {
            let prop_ref = queue.pop().unwrap();
            let deps = match self.document.get_property_dependants(&prop_ref.entity_id, &prop_ref.property_key) {
                Ok(deps) => deps,
                Err(_) => continue
            };
            for pr in deps {
                if ips.insert(pr.clone()) {
                    queue.push(pr.clone());
                }
            }
        }
        ips.into_iter().collect()
    }
    pub fn update(&mut self) {
        let t = time::get_time();
        let diff_time = t - self.prev_frame_time;
        for system in self.sub_systems.clone() {
            system.borrow_mut().update(self, diff_time);
        }
        while {
            let ae = mem::replace(&mut self.added_entities, vec![]);
            for e in ae {
                self.on_entity_added(&e);
            }
            let ips = self.build_property_cascades();
            self.on_property_value_change(&ips);
            self.changed_properties.len() > 0 || self.added_entities.len() > 0
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
    fn append_entity(&mut self, parent: &EntityId, type_name: &str, name: Option<String>) -> Result<EntityId, DocError> {
        match self.document.append_entity(parent.clone(), type_name, name) {
            Ok(entity_id) => {
                self.added_entities.push(entity_id);
                Ok(entity_id)
            },
            err @ _ => err
        }
    }
    fn get_entity_by_name(&self, name: &str) -> Option<EntityId> {
        self.document.get_entity_by_name(name)
    }
    fn set_property(&mut self, entity_id: &EntityId, property_key: &str, value: Pon) -> Result<(), DocError> {
        match self.document.set_property(entity_id, property_key, value) {
            Ok(_) => {
                self.changed_properties.insert(PropRef::new(entity_id, property_key));
                Ok(())
            },
            Err(err) => Err(err)
        }
    }
    fn get_property_value(&self, entity_id: &EntityId, property_key: &str) -> Result<Pon, DocError> {
        self.document.get_property_value(entity_id, property_key)
    }
    fn get_property_expression(&self, entity_id: &EntityId, property_key: &str) -> Result<&Pon, DocError> {
        self.document.get_property_expression(entity_id, property_key)
    }
    fn has_property(&self, entity_id: &EntityId, property_key: &str) -> Result<bool, DocError> {
        self.document.has_property(entity_id, property_key)
    }
    fn resolve_named_prop_ref(&self, entity_id: &EntityId, named_prop_ref: &NamedPropRef) -> Result<PropRef, DocError> {
        self.document.resolve_named_prop_ref(entity_id, named_prop_ref)
    }
    fn resolve_pon_dependencies(&self, entity_id: &EntityId, node: &Pon) -> Result<Pon, DocError> {
        self.document.resolve_pon_dependencies(entity_id, node)
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
