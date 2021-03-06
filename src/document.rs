
extern crate xml;

use pon::*;

use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::collections::hash_map::Entry;
use std::path::Path;
use std::io::Write;
use std::cell::RefCell;
use std::cell::Ref;
use std::any::Any;
use std::rc::Rc;

use xml::reader::EventReader;
use xml::reader::events::*;

#[derive(PartialEq, Debug, Clone)]
pub enum DocError {
    PonTranslateErr(PonTranslateErr),
    NoSuchProperty(String),
    NoSuchEntity(EntityId),
    CantFindEntityByName(String),
    InvalidParent
}

impl From<PonTranslateErr> for DocError {
    fn from(err: PonTranslateErr) -> DocError {
        DocError::PonTranslateErr(err)
    }
}

pub type EntityId = u64;

pub type EntityIter<'a> = Keys<'a, EntityId, Entity>;
pub type PropertyIter<'a> = Keys<'a, String, Property>;


#[derive(Debug)]
struct Property {
    expression: Rc<RefCell<Option<Pon>>>,
    dependants: Vec<PropRef>
}

#[derive(Debug)]
struct Entity {
    id: EntityId,
    type_name: String,
    properties: HashMap<String, Property>,
    name: Option<String>,
    children_ids: Vec<EntityId>,
    parent_id: Option<EntityId>
}

impl Entity {
    fn get_or_create_property(&mut self, property_key: &str) -> &mut Property {
        match self.properties.entry(property_key.to_string()) {
            Entry::Occupied(o) => {
                o.into_mut()
            },
            Entry::Vacant(v) => {
                v.insert(Property {
                    expression: Rc::new(RefCell::new(None)),
                    dependants: vec![]
                })
            }
        }
    }
}

pub struct Document {
    id_counter: EntityId,
    root: Option<EntityId>,
    entities: HashMap<EntityId, Entity>,
    entity_ids_by_name: HashMap<String, EntityId>,
    pub resources: HashMap<String, Box<Any>>,
    pub on_entity_added: Option<Box<Fn(&EntityId) -> ()>>,
    pub on_property_set: Option<Box<Fn(&EntityId, &str) -> ()>>
}

impl Document {
    pub fn new() -> Document {
        Document {
            id_counter: 0,
            root: None,
            entities: HashMap::new(),
            entity_ids_by_name: HashMap::new(),
            resources: HashMap::new(),
            on_entity_added: None,
            on_property_set: None
        }
    }
    fn new_id(&mut self) -> EntityId {
        self.id_counter += 1;
        return self.id_counter;
    }
    pub fn append_entity(&mut self, parent_id: Option<EntityId>, type_name: &str, name: Option<String>) -> Result<EntityId, DocError> {
        let id = self.new_id();
        let entity = Entity {
            id: id.clone(),
            type_name: type_name.to_string(),
            properties: HashMap::new(),
            name: name,
            parent_id: parent_id,
            children_ids: vec![]
        };
        if let Some(parent_id) = parent_id {
            let parent = match self.entities.get_mut(&parent_id) {
                Some(parent) => parent,
                None => return Err(DocError::InvalidParent)
            };
            parent.children_ids.push(id);
        } else {
            if self.root.is_some() {
                panic!("Cannot set root twice.");
            }
            self.root = Some(id);
        }
        if let &Some(ref name) = &entity.name {
            self.entity_ids_by_name.insert(name.clone(), entity.id);
        }
        self.entities.insert(entity.id, entity);
        if let &Some(ref cb) = &self.on_entity_added {
            cb(&id);
        }
        return Ok(id);
    }
    pub fn get_entity_by_name(&self, name: &str) -> Option<EntityId> {
        match self.entity_ids_by_name.get(&name.to_string()) {
            Some(id) => Some(id.clone()),
            None => None
        }
    }
    pub fn entities_iter(&self) -> EntityIter {
        self.entities.keys()
    }
    pub fn get_root(&self) -> Option<EntityId> {
        self.root.clone()
    }
    // returns all props that were invalidated
    pub fn set_property(&mut self, entity_id: &EntityId, property_key: &str, mut expression: Pon) -> Result<(), DocError> {
        //println!("set property {} {:?}", property_key, expression);
        let dependencies: Vec<PropRef> = {
            let entity = match self.entities.get(entity_id) {
                Some(entity) => entity,
                None => return Err(DocError::NoSuchEntity(*entity_id))
            };
            try!(self.build_property_node_dependencies(entity, &expression))
        };
        for PropRef { entity_id: dep_ent_id, property_key: dep_prop_key } in dependencies {
            match self.entities.get_mut(&dep_ent_id) {
                Some(dep_ent) => {
                    let prop_ref = PropRef { entity_id: entity_id.clone(), property_key: property_key.to_string() };
                    let mut prop = dep_ent.get_or_create_property(&dep_prop_key);
                    prop.dependants.push(prop_ref);
                },
                None => return Err(DocError::NoSuchEntity(dep_ent_id))
            }
        }
        {
            try!(self.resolve_pon_dependencies(&entity_id, &mut expression));
        }
        {
            let mut ent_mut = self.entities.get_mut(entity_id).unwrap();
            let prop = ent_mut.get_or_create_property(property_key);
            *prop.expression.borrow_mut() = Some(expression);
        }
        if let &Some(ref cb) = &self.on_property_set {
            cb(entity_id, property_key);
        }
        Ok(())
    }
    pub fn get_property(&self, entity_id: &EntityId, property_key: &str) -> Result<Ref<Pon>, DocError> {
        match self.entities.get(entity_id) {
            Some(entity) => self.get_entity_property(entity, property_key),
            None => Err(DocError::NoSuchEntity(*entity_id))
        }
    }
    pub fn has_property(&self, entity_id: &EntityId, name: &str) -> Result<bool, DocError> {
        match self.entities.get(entity_id) {
            Some(entity) => match entity.properties.get(name) {
                Some(prop) => Ok(prop.expression.borrow().is_some()),
                None => Ok(false)
            },
            None => Err(DocError::NoSuchEntity(*entity_id))
        }
    }
    pub fn get_properties(&self, entity_id: &EntityId) -> Result<Vec<PropRef>, DocError> {
        match self.entities.get(&entity_id) {
            Some(entity) => Ok(entity.properties.keys().map(|key| PropRef { entity_id: entity_id.clone(), property_key: key.clone() }).collect()),
            None => Err(DocError::NoSuchEntity(*entity_id))
        }
    }
    pub fn get_children(&self, entity_id: &EntityId) -> Result<&Vec<EntityId>, DocError> {
        match self.entities.get(&entity_id) {
            Some(entity) => Ok(&entity.children_ids),
            None => Err(DocError::NoSuchEntity(*entity_id))
        }
    }
    pub fn search_children(&self, entity_id: &EntityId, name: &str) -> Result<EntityId, DocError> {
        match self.entities.get(entity_id) {
            Some(entity) => {
                if let &Some(ref string) = &entity.name {
                    if string == name {
                        return Ok(entity.id);
                    }
                }
                for c in &entity.children_ids {
                    match self.search_children(&c, name) {
                        Ok(id) => return Ok(id),
                        _ => {}
                    }
                }
                Err(DocError::CantFindEntityByName(name.to_string()))
            },
            None => Err(DocError::NoSuchEntity(*entity_id))
        }
    }
    pub fn resolve_entity_path(&self, start_entity_id: &EntityId, path: &EntityPath) -> Result<EntityId, DocError> {
        match path {
            &EntityPath::This => Ok(*start_entity_id),
            &EntityPath::Parent => match self.entities.get(start_entity_id) {
                Some(entity) => Ok(entity.parent_id.unwrap().clone()),
                None => Err(DocError::NoSuchEntity(*start_entity_id))
            },
            &EntityPath::Named(ref name) => match self.entity_ids_by_name.get(name) {
                Some(entity_id) => Ok(entity_id.clone()),
                None => Err(DocError::CantFindEntityByName(name.to_string()))
            },
            &EntityPath::Search(ref path, ref search) => {
                match self.resolve_entity_path(start_entity_id, path) {
                    Ok(ent) => self.search_children(&ent, search),
                    Err(err) => Err(err)
                }
            }
        }
    }
    pub fn resolve_named_prop_ref(&self, start_entity_id: &EntityId, named_prop_ref: &NamedPropRef) -> Result<PropRef, DocError> {
        let owner_entity_id = try!(self.resolve_entity_path(start_entity_id, &named_prop_ref.entity_path));
        Ok(PropRef { entity_id: owner_entity_id, property_key: named_prop_ref.property_key.clone() })
    }
    pub fn get_entity_type_name(&self, entity_id: &EntityId) -> Result<&String, DocError> {
        match self.entities.get(&entity_id) {
            Some(entity) => Ok(&entity.type_name),
            None => Err(DocError::NoSuchEntity(*entity_id))
        }
    }

    pub fn from_file(path: &Path) -> Result<Document, DocError> {
        let mut doc = Document::new();
        let mut warnings = vec![];
        try!(doc.append_from_event_reader(&mut vec![], event_reader_from_file(path).events(), &mut warnings));
        if warnings.len() > 0 {
            println!("{} WARNINGS PARSING DOCUMENT:", warnings.len());
            println!("{}", warnings.join("\n"));
        }
        Ok(doc)
    }
    pub fn from_string(string: &str) -> Result<Document, DocError> {
        let mut doc = Document::new();
        let mut parser = EventReader::from_str(string);
        let mut warnings = vec![];
        try!(doc.append_from_event_reader(&mut vec![], parser.events(), &mut warnings));
        if warnings.len() > 0 {
            println!("{} WARNINGS PARSING DOCUMENT:", warnings.len());
            println!("{}", warnings.join("\n"));
        }
        Ok(doc)
    }


    fn build_property_node_dependencies(&self, entity: &Entity, node: &Pon) -> Result<Vec<PropRef>, DocError> {
        let mut named_refs = vec![];
        node.get_dependency_references(&mut named_refs);
        let mut refs = vec![];
        for named_prop_ref in named_refs {
            refs.push(try!(self.resolve_named_prop_ref(&entity.id, &named_prop_ref)));
        }
        return Ok(refs);
    }

    pub fn get_property_dependants(&self, entity_id: &EntityId, property_key: &str) -> Result<&Vec<PropRef>, DocError> {
        match self.entities.get(entity_id) {
            Some(entity) => match entity.properties.get(property_key) {
                Some(prop) => Ok(&prop.dependants),
                None => Err(DocError::NoSuchProperty(property_key.to_string()))
            },
            None => Err(DocError::NoSuchEntity(*entity_id))
        }
    }

    fn resolve_pon_dependencies(&mut self, entity_id: &EntityId, node: &mut Pon) -> Result<(), DocError> {
        match node {
            &mut Pon::TypedPon(box TypedPon { ref mut data, .. }) =>
                try!(self.resolve_pon_dependencies(entity_id, data)),
            &mut Pon::DependencyReference(ref named_prop_ref, ref mut resolved) => {
                let prop_ref = try!(self.resolve_named_prop_ref(&entity_id, &named_prop_ref));
                match self.entities.get_mut(&prop_ref.entity_id) {
                    Some(entity) => {
                        let prop = entity.get_or_create_property(&prop_ref.property_key);
                        *resolved = Some(ResolvedDependency {
                            prop_ref: prop_ref,
                            value: prop.expression.clone()
                        });
                    },
                    None => return Err(DocError::NoSuchEntity(prop_ref.entity_id))
                }
            },
            &mut Pon::Object(ref mut hm) => {
                for (_, v) in hm.iter_mut() {
                    try!(self.resolve_pon_dependencies(entity_id, v))
                }
            },
            &mut Pon::Array(ref mut arr) => {
                for v in arr.iter_mut() {
                    try!(self.resolve_pon_dependencies(entity_id, v))
                }
            },
            _ => {}
        };
        Ok(())
    }

    fn get_entity_property<'a>(&self, entity: &'a Entity, property_key: &str) -> Result<Ref<'a, Pon>, DocError> {
        match entity.properties.get(property_key) {
            Some(property) => match Ref::filter_map(property.expression.borrow(), |x| match x {
                &Some(ref r) => Some(r),
                &None => None
            }) {
                Some(expression) => Ok(expression),
                None => Err(DocError::NoSuchProperty(property_key.to_string()))
            },
            None => Err(DocError::NoSuchProperty(property_key.to_string()))
        }
    }

    fn append_from_event_reader<T: Iterator<Item=XmlEvent>>(&mut self, mut entity_stack: &mut Vec<EntityId>, mut events: T, warnings: &mut Vec<String>) -> Result<(), DocError> {
        while let Some(e) = events.next() {
            match e {
                XmlEvent::StartElement { name: type_name, attributes, .. } => {
                    let entity_name = match attributes.iter().find(|x| x.name.local_name == "name") {
                        Some(attr) => Some(attr.value.to_string()),
                        None => None
                    };
                    let parent = match entity_stack.last() {
                        Some(parent) => Some(*parent),
                        None => None
                    };
                    let entity_id = match self.append_entity(parent, &type_name.local_name, entity_name) {
                        Ok(id) => id,
                        Err(err) => {
                            warnings.push(format!("Failed to append entity {:?}: {:?}", type_name.local_name, err));
                            continue;
                        }
                    };

                    for attribute in attributes {
                        if attribute.name.local_name == "name" { continue; }
                        match Pon::from_string(&attribute.value) {
                            Ok(node) => match self.set_property(&entity_id, &attribute.name.local_name, node) {
                                Ok(_) => {},
                                Err(err) => warnings.push(format!("Failed to set property {} for entity {:?}: {:?}", attribute.name.local_name, type_name.local_name, err))
                            },
                            Err(err) => warnings.push(format!("Error parsing property {} of entity {:?}: {} with error: {:?}", attribute.name.local_name, type_name.local_name, attribute.value, err))
                        };
                    }
                    entity_stack.push(entity_id);
                }
                XmlEvent::EndElement { .. } => {
                    entity_stack.pop();
                }
                XmlEvent::Error(e) => {
                    warnings.push(format!("Xml parsing error: {}", e));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn entity_to_xml<T: Write>(&self, entity_id: &EntityId, writer: &mut xml::writer::EventWriter<T>) {
        let entity = self.entities.get(entity_id).unwrap();
        let type_name = xml::name::Name::local(&entity.type_name);
        let mut attrs: Vec<xml::attribute::OwnedAttribute> = entity.properties.iter().filter_map(|(name, prop)| {
            match &*prop.expression.borrow() {
                &Some(ref expression) => Some(xml::attribute::OwnedAttribute {
                    name: xml::name::OwnedName::local(name.to_string()),
                    value: expression.to_string()
                }),
                &None => None
            }
        }).collect();
        if let &Some(ref name) = &entity.name {
            attrs.push(xml::attribute::OwnedAttribute {
                name: xml::name::OwnedName::local("name"),
                value: name.to_string()
            });
        }
        attrs.sort_by(|a, b| a.name.local_name.cmp(&b.name.local_name) );
        writer.write(xml::writer::events::XmlEvent::StartElement {
            name: type_name.clone(),
            attributes: attrs.iter().map(|x| x.borrow()).collect(),
            namespace: &xml::namespace::Namespace::empty()
        }).unwrap();
        for e in &entity.children_ids {
            self.entity_to_xml(e, writer);
        }
        writer.write(xml::writer::events::XmlEvent::EndElement {
            name: type_name.clone()
        }).unwrap();
    }
    fn to_xml(&self) -> String {
        let mut buff = vec![];
        {
            let mut writer = xml::writer::EventWriter::new(&mut buff);
            writer.write(xml::writer::events::XmlEvent::StartDocument {
                version: xml::common::XmlVersion::Version11,
                encoding: None,
                standalone: None
            }).unwrap();
            if self.root.is_some() {
                self.entity_to_xml(&self.root.unwrap(), &mut writer);
            }
        }
        String::from_utf8(buff).unwrap()
    }
}

fn event_reader_from_file(path: &Path) -> EventReader<BufReader<File>> {
    let file = File::open(path).unwrap();
    let file = BufReader::new(file);

    EventReader::new(file)
}

impl ToString for Document {
    fn to_string(&self) -> String {
        self.to_xml()
    }
}


#[test]
fn test_property_get() {
    let doc = Document::from_string(r#"<Entity name="tmp" x="5.0" />"#).unwrap();
    let ent = doc.get_entity_by_name("tmp").unwrap();
    assert_eq!(*doc.get_property(&ent, "x").unwrap(), Pon::from_string("5.0").unwrap());
}

#[test]
fn test_property_set() {
    let mut doc = Document::from_string(r#"<Entity name="tmp" x="5.0" />"#).unwrap();
    let ent = doc.get_entity_by_name("tmp").unwrap();
    {
        doc.set_property(&ent, "x", Pon::Integer(9)).unwrap();
    }
    assert_eq!(*doc.get_property(&ent, "x").unwrap(), Pon::from_string("9").unwrap());
}

#[test]
fn test_property_reference_straight() {
    let doc = Document::from_string(r#"<Entity name="tmp" x="5.0" y="@this.x" />"#).unwrap();
    let ent = doc.get_entity_by_name("tmp").unwrap();
    assert_eq!(doc.get_property(&ent, "y").unwrap().concretize().unwrap(), Pon::from_string("5.0").unwrap());
}

#[test]
fn test_property_reference_object() {
    let doc = Document::from_string(r#"<Entity name="tmp" x="5.0" y="{ some: @this.x }" />"#).unwrap();
    let ent = doc.get_entity_by_name("tmp").unwrap();
    assert_eq!(doc.get_property(&ent, "y").unwrap().concretize().unwrap(), Pon::from_string("{ some: 5.0 }").unwrap());
}

#[test]
fn test_property_reference_transfer() {
    let doc = Document::from_string(r#"<Entity name="tmp" x="5.0" y="something @this.x" />"#).unwrap();
    let ent = doc.get_entity_by_name("tmp").unwrap();
    assert_eq!(doc.get_property(&ent, "y").unwrap().concretize().unwrap(), Pon::from_string("something 5.0").unwrap());
}

#[test]
fn test_property_reference_array() {
    let doc = Document::from_string(r#"<Entity name="tmp" x="5.0" y="[@this.x]" />"#).unwrap();
    let ent = doc.get_entity_by_name("tmp").unwrap();
    assert_eq!(doc.get_property(&ent, "y").unwrap().concretize().unwrap(),
        Pon::from_string("[5.0]").unwrap());
}

#[test]
fn test_property_reference_bad_ref() {
    let doc = Document::from_string(r#"<Entity name="tmp" x="5.0" y="@what.x" />"#).unwrap();
    let ent = doc.get_entity_by_name("tmp").unwrap();
    assert_eq!(doc.get_property(&ent, "y").err().unwrap(), DocError::NoSuchProperty("y".to_string()));
}

#[test]
fn test_property_reference_parent() {
    let doc = Document::from_string(r#"<Entity x="5.0"><Entity name="tmp" y="@parent.x" /></Entity>"#).unwrap();
    let ent = doc.get_entity_by_name("tmp").unwrap();
    assert_eq!(doc.get_property(&ent, "y").unwrap().concretize().unwrap(), Pon::Float(5.0));
}

#[test]
fn test_property_reference_update() {
    let mut doc = Document::from_string(r#"<Entity name="tmp" x="5.0" y="@this.x" />"#).unwrap();
    let ent = doc.get_entity_by_name("tmp").unwrap();
    {
        doc.set_property(&ent, "x", Pon::Integer(9)).ok().unwrap();
    }
    assert_eq!(doc.get_property(&ent, "y").unwrap().concretize().unwrap(), Pon::from_string("9").unwrap());
}


#[test]
fn test_property_reference_not_yet_created() {
    let mut doc = Document::from_string(r#"<Entity name="tmp" y="@this.x" />"#).unwrap();
    let ent = doc.get_entity_by_name("tmp").unwrap();
    {
        doc.set_property(&ent, "x", Pon::Float(9.0)).ok().unwrap();
    }
    assert_eq!(doc.get_property(&ent, "y").unwrap().concretize().unwrap(), Pon::Float(9.0));
}


#[test]
fn test_document_to_string_empty() {
    let doc = Document::new();
    assert_eq!(doc.to_string(), "<?xml version=\"1.1\" encoding=\"UTF-8\"?>");
}
