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
    pub arg: PropNode
}

#[derive(PartialEq, Debug, Clone)]
pub enum PropNode {
    PropTransform(Box<PropTransform>),
    DependencyReference(NamedPropRef),
    Reference(NamedPropRef),
    Array(Vec<PropNode>),
    Object(HashMap<String, PropNode>),
    Float(f32),
    Integer(i64),
    String(String),
    Nil
}

impl Hash for PropNode {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        let str = format!("{:?}", self);
        str.hash(state);
    }
}

impl Eq for PropNode {
    // This "works" because it derives PartialEq, so there's already an Eq method on it
}

#[derive(PartialEq, Debug, Clone)]
pub enum PropTranslateErr {
    MismatchType { expected: String, found: String },
    NoSuchField { field: String },
    UnrecognizedPropTransform(String),
    Generic(String)
}

impl PropNode {
    pub fn get_dependency_references(&self, references: &mut Vec<NamedPropRef>) {
        match self {
            &PropNode::PropTransform(box PropTransform { ref name, ref arg } ) =>
                arg.get_dependency_references(references),
            &PropNode::DependencyReference(ref reference) => {
                references.push(reference.clone());
            },
            &PropNode::Object(ref hm) => {
                for (k, v) in hm {
                    v.get_dependency_references(references);
                }
            },
            &PropNode::Array(ref arr) => {
                for v in arr {
                    v.get_dependency_references(references);
                }
            },
            _ => {}
        }
    }
    pub fn as_transform(&self) -> Result<&PropTransform, PropTranslateErr> {
        match self {
            &PropNode::PropTransform(box ref transform) => Ok(transform),
            _ => Err(PropTranslateErr::MismatchType { expected: "PropTransform".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_float(&self) -> Result<&f32, PropTranslateErr> {
        match self {
            &PropNode::Float(ref v) => Ok(v),
            _ => Err(PropTranslateErr::MismatchType { expected: "Float".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_integer(&self) -> Result<&i64, PropTranslateErr> {
        match self {
            &PropNode::Integer(ref v) => Ok(v),
            _ => Err(PropTranslateErr::MismatchType { expected: "Integer".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_string(&self) -> Result<&String, PropTranslateErr> {
        match self {
            &PropNode::String(ref v) => Ok(v),
            _ => Err(PropTranslateErr::MismatchType { expected: "String".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_reference(&self) -> Result<&NamedPropRef, PropTranslateErr> {
        match self {
            &PropNode::Reference(ref v) => Ok(v),
            _ => Err(PropTranslateErr::MismatchType { expected: "Reference".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_object(&self) -> Result<&HashMap<String, PropNode>, PropTranslateErr> {
        match self {
            &PropNode::Object(ref hm) => Ok(hm),
            _ => Err(PropTranslateErr::MismatchType { expected: "Object".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_array(&self) -> Result<&Vec<PropNode>, PropTranslateErr> {
        match self {
            &PropNode::Array(ref arr) => Ok(arr),
            _ => Err(PropTranslateErr::MismatchType { expected: "Array".to_string(), found: format!("{:?}", self) })
        }
    }
    pub fn as_float_array(&self) -> Result<Vec<f32>, PropTranslateErr> {
        let arr = try!(self.as_array());
        let mut res_arr = vec![];
        for v in arr {
            res_arr.push(*try!(v.as_float()));
        }
        return Ok(res_arr);
    }
    pub fn as_integer_array(&self) -> Result<Vec<i64>, PropTranslateErr> {
        let arr = try!(self.as_array());
        let mut res_arr = vec![];
        for v in arr {
            res_arr.push(*try!(v.as_integer()));
        }
        return Ok(res_arr);
    }
}
