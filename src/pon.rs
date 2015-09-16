peg_file! pon_peg("pon.rustpeg");

pub use pon::pon_peg::ParseError as PonParseError;

use document::EntityId;
pub use pon_translations::*;

use std::collections::HashMap;
use std::slice::SliceConcatExt;
use std::hash::Hasher;
use std::hash::Hash;
use std::cmp::Eq;
use cgmath;
use std::intrinsics;
use std::ops::Deref;
use std::rc::Rc;
use std::cell::{RefCell,Ref};

#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub enum EntityPath {
    This,
    Parent,
    Named(String),
    Search(Box<EntityPath>, String)
}
impl ToString for EntityPath {
    fn to_string(&self) -> String {
        match self {
            &EntityPath::This => "this".to_string(),
            &EntityPath::Parent => "parent".to_string(),
            &EntityPath::Named(ref name) => name.to_string(),
            &EntityPath::Search(ref path, ref search) => format!("{}:{}", path.to_string(), search),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct NamedPropRef {
    pub entity_path: EntityPath,
    pub property_key: String
}
impl NamedPropRef {
    pub fn new(entity_path: EntityPath, property_key: &str) -> NamedPropRef {
        NamedPropRef {
            entity_path: entity_path,
            property_key: property_key.to_string()
        }
    }
}
impl ToString for NamedPropRef {
    fn to_string(&self) -> String {
        format!("{}.{}", self.entity_path.to_string(), self.property_key)
    }
}

#[derive(PartialEq, Debug, Clone, Hash)]
pub struct PropRef {
    pub entity_id: EntityId,
    pub property_key: String
}
impl PropRef {
    pub fn new(entity_id: &EntityId, property_key: &str) -> PropRef {
        PropRef {
            entity_id: *entity_id,
            property_key: property_key.to_string()
        }
    }
}
impl Eq for PropRef {
    // hack, relies on PartialEq to be defined
}

#[derive(PartialEq, Debug, Clone)]
pub struct TypedPon {
    pub type_name: String,
    pub data: Pon
}
impl ToString for TypedPon {
    fn to_string(&self) -> String {
        format!("{} {}", self.type_name.to_string(), self.data.to_string())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ResolvedDependency {
    pub prop_ref: PropRef,
    pub value: Rc<RefCell<Option<Pon>>>
}

#[derive(PartialEq, Debug, Clone)]
pub enum Pon {
    TypedPon(Box<TypedPon>),
    DependencyReference(NamedPropRef, Option<ResolvedDependency>),
    Reference(NamedPropRef),
    Array(Vec<Pon>),
    FloatArray(Vec<f32>),
    IntegerArray(Vec<i64>),
    Object(HashMap<String, Pon>),
    Float(f32),
    Integer(i64),
    String(String),
    Boolean(bool),
    Vector3(cgmath::Vector3<f32>),
    Vector4(cgmath::Vector4<f32>),
    Matrix4(cgmath::Matrix4<f32>),
    Nil
}


impl Pon {
    pub fn from_string(string: &str) -> Result<Pon, PonParseError> {
        pon_peg::body(string)
    }
    pub fn new_typed_pon(type_name: &str, data: Pon) -> Pon {
        Pon::TypedPon(Box::new(TypedPon { type_name: type_name.to_string(), data: data }))
    }
    pub fn get_dependency_references(&self, references: &mut Vec<NamedPropRef>) {
        match self {
            &Pon::TypedPon(box TypedPon { ref data, .. } ) =>
                data.get_dependency_references(references),
            &Pon::DependencyReference(ref reference, _) => {
                references.push(reference.clone());
            },
            &Pon::Object(ref hm) => {
                for (_, v) in hm {
                    v.get_dependency_references(references);
                }
            },
            &Pon::Array(ref arr) => {
                for v in arr {
                    v.get_dependency_references(references);
                }
            },
            _ => {}
        }
    }
    pub fn translate<T: 'static>(&self, context: &mut TranslateContext) -> Result<T, PonTranslateErr> where Pon: Translatable<T> {
        match self {
            &Pon::DependencyReference(ref named_prop_ref, ref dep) => match dep {
                &Some(ref dep) => match &*dep.value.borrow() {
                    &Some(ref pon) => pon.translate(context),
                    &None => return Err(PonTranslateErr::ReferenceToNonExistentProperty(named_prop_ref.clone()))
                },
                &None => panic!("Trying to translate on non-resolved dependency reference")
            },
            _ => match self.inner_translate(context) {
                Ok(val) => Ok(val),
                Err(err) => {
                    let type_name = unsafe {
                        ::std::intrinsics::type_name::<T>()
                    };
                    Err(PonTranslateErr::InnerError { in_pon: self.clone(), error: Box::new(err), trying_to_translate_to: type_name.to_string() })
                }
            }
        }
    }

    pub fn field(&self, field: &str) -> Result<&Pon, PonTranslateErr> {
        match self {
            &Pon::Object(ref value) => match value.get(field) {
                Some(value) => Ok(value),
                None => Err(PonTranslateErr::NoSuchField { field: field.to_string() })
            },
            _ => Err(PonTranslateErr::MismatchType { expected: "Object".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn field_as<T: 'static>(&self, field: &str, context: &mut TranslateContext) -> Result<T, PonTranslateErr> where Pon: Translatable<T> {
        try!(self.field(field)).translate(context)
    }
    pub fn field_as_or<T: 'static>(&self, field: &str, or: T, context: &mut TranslateContext) -> Result<T, PonTranslateErr> where Pon: Translatable<T> {
        match self.field(field) {
            Ok(val) => val.translate(context),
            Err(PonTranslateErr::NoSuchField { .. }) => Ok(or),
            Err(err) => Err(err)
        }
    }

    pub fn concretize(&self) -> Result<Pon, PonTranslateErr> {
        self.as_resolved(|pon| {
            match pon {
               &Pon::TypedPon(box TypedPon { ref type_name, ref data }) =>
                   Ok(Pon::TypedPon(Box::new(TypedPon {
                       type_name: type_name.clone(),
                       data: try!(data.concretize())
                   }))),
               &Pon::Object(ref hm) => Ok(Pon::Object(hm.iter().map(|(k,v)| {
                       (k.clone(), v.concretize().unwrap())
                   }).collect())),
               &Pon::Array(ref arr) => Ok(Pon::Array(arr.iter().map(|v| {
                        v.concretize().unwrap()
                   }).collect())),
               _ => Ok(pon.clone())
           }
        })
    }

    pub fn as_resolved<'a, T: 'static, F: FnOnce(&Pon) -> Result<T, PonTranslateErr> + 'a>(&'a self, func: F) -> Result<T, PonTranslateErr> {
        match self {
            &Pon::DependencyReference(ref named_prop_ref, Some(ref resolved)) => {
                match &*resolved.value.borrow() {
                    &Some(ref v) => v.as_resolved(func),
                    &None => return Err(PonTranslateErr::ReferenceToNonExistentProperty(named_prop_ref.clone()))
                }
            },
            &Pon::DependencyReference(_, None) => panic!("Cannot treat non-resolved pon as resolved."),
            _ => func(self),
        }
    }

    pub fn as_typed<'a, T: 'static, F: FnOnce(&TypedPon) -> Result<T, PonTranslateErr> + 'a>(&'a self, func: F) -> Result<T, PonTranslateErr> {
        self.as_resolved(|pon| match pon {
            &Pon::TypedPon(box ref value) => func(value),
            _ => Err(PonTranslateErr::MismatchType { expected: "TypedPon".to_string(), found: format!("{:?}", pon) })
        })
    }
    pub fn as_array<'a, T: 'static, F: FnOnce(&Vec<Pon>) -> Result<T, PonTranslateErr> + 'a>(&'a self, func: F) -> Result<T, PonTranslateErr> {
        self.as_resolved(|pon| match pon {
            &Pon::Array(ref value) => func(value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Array".to_string(), found: format!("{:?}", pon) })
        })
    }
    pub fn as_object<'a, T: 'static, F: FnOnce(&HashMap<String, Pon>) -> Result<T, PonTranslateErr> + 'a>(&'a self, func: F) -> Result<T, PonTranslateErr> {
        self.as_resolved(|pon| match pon {
            &Pon::Object(ref value) => func(value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Object".to_string(), found: format!("{:?}", pon) })
        })
    }
    // pub fn as_float_array(&self) -> Result<Vec<f32>, PonTranslateErr> {
    //     self.as_resolved(|pon| {
    //         match pon {
    //             &Pon::Array(ref arr) => {
    //                 let mut res_arr = vec![];
    //                 for v in arr {
    //                     res_arr.push(try!(v.as_float()));
    //                 }
    //                 return Ok(res_arr);
    //             },
    //             &Pon::FloatArray(ref value) => Ok(value.clone()),
    //             _ => Err(PonTranslateErr::MismatchType { expected: "Array or FloatArray".to_string(), found: format!("{:?}", self) })
    //         }
    //     })
    // }
    // pub fn as_float(&self) -> Result<f32, PonTranslateErr> {
    //     self.as_resolved(|pon| {
    //         match pon {
    //             &Pon::Float(ref value) => Ok(value.clone()),
    //             _ => Err(PonTranslateErr::MismatchType { expected: "Float".to_string(), found: format!("{:?}", self) })
    //         }
    //     })
    // }

    pub fn as_reference(&self) -> Result<&NamedPropRef, PonTranslateErr> {
        match self {
            &Pon::Reference(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Reference".to_string(), found: format!("{:?}", self) })
        }
    }
}

impl ToString for Pon {
    fn to_string(&self) -> String {
        match self {
            &Pon::TypedPon(box ref typed_pon) => typed_pon.to_string(),
            &Pon::DependencyReference(ref named_prop_ref, _) => format!("@{}", named_prop_ref.to_string()),
            &Pon::Reference(ref named_prop_ref) => format!("{}", named_prop_ref.to_string()),
            &Pon::Array(ref array) => {
                let a: Vec<String> = array.iter().map(|x| x.to_string()).collect();
                let mut s = a.join(", ");
                if s.len() > 120 { s = a.join(",\n"); }
                format!("[{}]", s)
            },
            &Pon::FloatArray(ref array) => array.to_pon().to_string(),
            &Pon::IntegerArray(ref array) => array.to_pon().to_string(),
            &Pon::Object(ref hm) => {
                let a: Vec<String> = hm.iter().map(|(k, v)| format!("{}: {}", k.to_string(), v.to_string())).collect();
                let mut s = a.join(", ");
                if s.len() > 120 { s = a.join(",\n"); }
                format!("{{ {} }}", s)
            },
            &Pon::Float(ref v) => format!("{:.10}", v),
            &Pon::Integer(ref v) => v.to_string(),
            &Pon::String(ref v) => format!("'{}'", v),
            &Pon::Boolean(ref v) => format!("{}", v),
            &Pon::Vector3(ref v) => v.to_pon().to_string(),
            &Pon::Vector4(ref v) => v.to_pon().to_string(),
            &Pon::Matrix4(ref v) => v.to_pon().to_string(),
            &Pon::Nil => "()".to_string()
        }
    }
}

impl Hash for Pon {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        let str = format!("{:?}", self);
        str.hash(state);
    }
}

impl Eq for Pon {
    // This "works" because it derives PartialEq, so there's already an Eq method on it
}


pub trait ToPon {
    fn to_pon(&self) -> Pon;
}

impl ToPon for Pon {
    fn to_pon(&self) -> Pon {
        self.clone()
    }
}
impl ToPon for f32 {
    fn to_pon(&self) -> Pon {
        Pon::Float(*self)
    }
}
impl ToPon for Vec<f32> {
    fn to_pon(&self) -> Pon {
        Pon::Array(self.iter().map(|v| Pon::Float(*v)).collect())
    }
}
impl ToPon for Vec<i64> {
    fn to_pon(&self) -> Pon {
        Pon::Array(self.iter().map(|v| Pon::Integer(*v)).collect())
    }
}
