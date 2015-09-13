
use std::mem;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;

use document::*;
use pon::*;
use interface::*;

pub struct System {
    document: Document,
    sub_systems: Vec<Rc<RefCell<Box<ISubSystem>>>>,
    changed_properties: Rc<RefCell<HashSet<PropRef>>>,
    added_entities: Rc<RefCell<Vec<EntityId>>>,
    pub running: bool
}

impl System {
    pub fn new() -> System {
        let pyramid = System {
            document: Document::new(),
            sub_systems: vec![],
            changed_properties: Rc::new(RefCell::new(HashSet::new())),
            added_entities: Rc::new(RefCell::new(vec![])),
            running: true
        };
        return pyramid;
    }
    pub fn add_subsystem(&mut self, subsystem: Box<ISubSystem>) {
        self.sub_systems.push(Rc::new(RefCell::new(subsystem)));
    }
    pub fn set_document(&mut self, document: Document) {
        self.document = document;
        let added_entities = self.added_entities.clone();
        self.document.on_entity_added = Some(Box::new(move |entity_id| {
            added_entities.borrow_mut().push(*entity_id);
        }));
        let changed_properties = self.changed_properties.clone();
        self.document.on_property_set = Some(Box::new(move |entity_id, property_key| {
            changed_properties.borrow_mut().insert(PropRef::new(entity_id, property_key));
        }));
        for system in self.sub_systems.clone() {
            system.borrow_mut().on_document_loaded(self);
        }
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }
    pub fn exit(&mut self) {
        self.running = false;
    }
    fn build_property_cascades(&mut self) -> Vec<PropRef> {
        let mut ips = mem::replace(&mut *self.changed_properties.borrow_mut(), HashSet::new());
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
        for system in self.sub_systems.clone() {
            system.borrow_mut().update(self);
        }
        while {
            let ae = mem::replace(&mut *self.added_entities.borrow_mut(), vec![]);
            for e in ae {
                self.on_entity_added(&e);
            }
            let ips = self.build_property_cascades();
            self.on_property_value_change(&ips);
            self.changed_properties.borrow().len() > 0 || self.added_entities.borrow().len() > 0
        } {};
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
