use document::EntityId;

use std::collections::HashMap;
use std::hash::Hasher;
use std::hash::Hash;
use std::cmp::Eq;

#[derive(PartialEq, Debug, Clone)]
pub struct NamedPropRef {
    pub entity_name: String,
    pub property_key: String
}

#[derive(PartialEq, Debug, Clone)]
pub struct PropRef {
    pub entity_id: EntityId,
    pub property_key: String
}

#[derive(PartialEq, Debug, Clone)]
pub struct PropTransform {
    pub name: String,
    pub arg: Pon
}

#[derive(PartialEq, Debug, Clone)]
pub enum Pon {
    PropTransform(Box<PropTransform>),
    DependencyReference(NamedPropRef),
    Reference(NamedPropRef),
    Array(Vec<Pon>),
    FloatArray(Vec<f32>),
    IntegerArray(Vec<i64>),
    Object(HashMap<String, Pon>),
    Float(f32),
    Integer(i64),
    String(String),
    Nil
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

#[derive(PartialEq, Debug, Clone)]
pub enum PropTranslateErr {
    MismatchType { expected: String, found: String },
    NoSuchField { field: String },
    UnrecognizedPropTransform(String),
    Generic(String)
}

impl Pon {
    pub fn get_dependency_references(&self, references: &mut Vec<NamedPropRef>) {
        match self {
            &Pon::PropTransform(box PropTransform { ref name, ref arg } ) =>
                arg.get_dependency_references(references),
            &Pon::DependencyReference(ref reference) => {
                references.push(reference.clone());
            },
            &Pon::Object(ref hm) => {
                for (k, v) in hm {
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
    pub fn as_transform(&self) -> Result<&PropTransform, PropTranslateErr> {
        match self {
            &Pon::PropTransform(box ref transform) => Ok(transform),
            _ => Err(PropTranslateErr::MismatchType { expected: "PropTransform".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_float(&self) -> Result<&f32, PropTranslateErr> {
        match self {
            &Pon::Float(ref v) => Ok(v),
            _ => Err(PropTranslateErr::MismatchType { expected: "Float".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_integer(&self) -> Result<&i64, PropTranslateErr> {
        match self {
            &Pon::Integer(ref v) => Ok(v),
            _ => Err(PropTranslateErr::MismatchType { expected: "Integer".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_string(&self) -> Result<&String, PropTranslateErr> {
        match self {
            &Pon::String(ref v) => Ok(v),
            _ => Err(PropTranslateErr::MismatchType { expected: "String".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_reference(&self) -> Result<&NamedPropRef, PropTranslateErr> {
        match self {
            &Pon::Reference(ref v) => Ok(v),
            _ => Err(PropTranslateErr::MismatchType { expected: "Reference".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_object(&self) -> Result<&HashMap<String, Pon>, PropTranslateErr> {
        match self {
            &Pon::Object(ref hm) => Ok(hm),
            _ => Err(PropTranslateErr::MismatchType { expected: "Object".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn get_object_field(&self, field: &str) -> Result<&Pon, PropTranslateErr> {
        let obj = try!(self.as_object());
        match obj.get(field) {
            Some(ref value) => Ok(value),
            None => Err(PropTranslateErr::NoSuchField { field: field.to_string() })
        }
    }
    pub fn as_array(&self) -> Result<&Vec<Pon>, PropTranslateErr> {
        match self {
            &Pon::Array(ref arr) => Ok(arr),
            _ => Err(PropTranslateErr::MismatchType { expected: "Array".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_float_array(&self) -> Result<Vec<f32>, PropTranslateErr> {
        match self {
            &Pon::Array(ref arr) => {
                let mut res_arr = vec![];
                for v in arr {
                    res_arr.push(*try!(v.as_float()));
                }
                return Ok(res_arr);
            },
            &Pon::FloatArray(ref arr) => Ok(arr.clone()),
            _ => Err(PropTranslateErr::MismatchType { expected: "Array or FloatArray".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_integer_array(&self) -> Result<Vec<i64>, PropTranslateErr> {
        match self {
            &Pon::Array(ref arr) => {
                let mut res_arr = vec![];
                for v in arr {
                    res_arr.push(*try!(v.as_integer()));
                }
                return Ok(res_arr);
            },
            &Pon::IntegerArray(ref arr) => Ok(arr.clone()),
            _ => Err(PropTranslateErr::MismatchType { expected: "Array or IntegerArray".to_string(), found: format!("{:?}", self) })
        }
    }
}
