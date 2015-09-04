peg_file! pon_peg("pon.rustpeg");

pub use pon::pon_peg::ParseError as PonParseError;

use document::EntityId;

use std::collections::HashMap;
use std::hash::Hasher;
use std::hash::Hash;
use std::cmp::Eq;
use std::borrow::Cow;
use cgmath;

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
pub struct TypedPon {
    pub type_name: String,
    pub data: Pon
}

#[derive(PartialEq, Debug, Clone)]
pub enum Pon {
    TypedPon(Box<TypedPon>),
    DependencyReference(NamedPropRef),
    Reference(NamedPropRef),
    Array(Vec<Pon>),
    FloatArray(Vec<f32>),
    IntegerArray(Vec<i64>),
    Object(HashMap<String, Pon>),
    Float(f32),
    Integer(i64),
    String(String),
    Vector3(cgmath::Vector3<f32>),
    Nil
}

#[derive(PartialEq, Debug, Clone)]
pub enum PonTranslateErr {
    MismatchType { expected: String, found: String },
    NoSuchField { field: String },
    InvalidValue { value: String },
    UnrecognizedType(String),
    InnerError { in_pon: Pon, error: Box<PonTranslateErr> },
    Generic(String)
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


pub trait Translatable<'a, T> {
    fn inner_translate(&'a self) -> Result<T, PonTranslateErr>;
}

impl<'a> Translatable<'a, &'a TypedPon> for Pon {
    fn inner_translate(&'a self) -> Result<&'a TypedPon, PonTranslateErr> {
        match self {
            &Pon::TypedPon(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "TypedPon".to_string(), found: format!("{:?}", self) })
        }
    }
}

impl<'a> Translatable<'a, &'a f32> for Pon {
    fn inner_translate(&'a self) -> Result<&'a f32, PonTranslateErr> {
        match self {
            &Pon::Float(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Float".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a> Translatable<'a, f32> for Pon {
    fn inner_translate(&'a self) -> Result<f32, PonTranslateErr> {
        match self {
            &Pon::Float(ref value) => Ok(*value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Float".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a> Translatable<'a, &'a i64> for Pon {
    fn inner_translate(&'a self) -> Result<&'a i64, PonTranslateErr> {
        match self {
            &Pon::Integer(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Integer".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a> Translatable<'a, i64> for Pon {
    fn inner_translate(&'a self) -> Result<i64, PonTranslateErr> {
        match self {
            &Pon::Integer(ref value) => Ok(*value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Integer".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a> Translatable<'a, &'a str> for Pon {
    fn inner_translate(&'a self) -> Result<&'a str, PonTranslateErr> {
        match self {
            &Pon::String(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "String".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a> Translatable<'a, Cow<'a, Vec<f32>>> for Pon {
    fn inner_translate(&'a self) -> Result<Cow<'a, Vec<f32>>, PonTranslateErr> {
        match self {
            &Pon::Array(ref arr) => {
                let mut res_arr = vec![];
                for v in arr {
                    res_arr.push(*try!(v.as_float()));
                }
                return Ok(Cow::Owned(res_arr));
            },
            &Pon::FloatArray(ref value) => Ok(Cow::Borrowed(&value)),
            _ => Err(PonTranslateErr::MismatchType { expected: "Array or FloatArray".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a> Translatable<'a, Cow<'a, Vec<i64>>> for Pon {
    fn inner_translate(&'a self) -> Result<Cow<'a, Vec<i64>>, PonTranslateErr> {
        match self {
            &Pon::Array(ref arr) => {
                let mut res_arr = vec![];
                for v in arr {
                    res_arr.push(*try!(v.as_integer()));
                }
                return Ok(Cow::Owned(res_arr));
            },
            &Pon::IntegerArray(ref value) => Ok(Cow::Borrowed(&value)),
            _ => Err(PonTranslateErr::MismatchType { expected: "Array or IntegerArray".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a> Translatable<'a, &'a Vec<Pon>> for Pon {
    fn inner_translate(&'a self) -> Result<&'a Vec<Pon>, PonTranslateErr> {
        match self {
            &Pon::Array(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Array".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a> Translatable<'a, &'a HashMap<String, Pon>> for Pon {
    fn inner_translate(&'a self) -> Result<&'a HashMap<String, Pon>, PonTranslateErr> {
        match self {
            &Pon::Object(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Object".to_string(), found: format!("{:?}", self) })
        }
    }
}


impl<'a, T> Translatable<'a, Vec<T>> for Pon where Pon: Translatable<'a, T> {
    fn inner_translate(&'a self) -> Result<Vec<T>, PonTranslateErr> {
        let source: &Vec<Pon> = try!(self.translate::<&Vec<Pon>>());
        let mut out = vec![];
        for v in source {
            out.push(From::from(try!(v.translate())));
        }
        Ok(out)
    }
}



#[test]
fn test_translate_integer() {
    let node = Pon::Integer(5);
    let i: &i64 = node.translate().unwrap();
    assert_eq!(*i, 5);
}


impl Pon {
    pub fn from_string(string: &str) -> Result<Pon, PonParseError> {
        pon_peg::body(string)
    }
    pub fn get_dependency_references(&self, references: &mut Vec<NamedPropRef>) {
        match self {
            &Pon::TypedPon(box TypedPon { ref data, .. } ) =>
                data.get_dependency_references(references),
            &Pon::DependencyReference(ref reference) => {
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
    pub fn translate<'a, T>(&'a self) -> Result<T, PonTranslateErr> where Pon: Translatable<'a, T> {
        match self.inner_translate() {
            Ok(val) => Ok(val),
            Err(err) => Err(PonTranslateErr::InnerError { in_pon: self.clone(), error: Box::new(err) })
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
    pub fn field_as<'a, T>(&'a self, field: &'a str) -> Result<T, PonTranslateErr> where Pon: Translatable<'a, T> {
        try!(self.field(field)).translate()
    }
    pub fn field_as_or<'a, T>(&'a self, field: &'a str, or: T) -> Result<T, PonTranslateErr> where Pon: Translatable<'a, T> {
        match self.field(field) {
            Ok(val) => val.translate(),
            Err(PonTranslateErr::NoSuchField { .. }) => Ok(or),
            Err(err) => Err(err)
        }
    }

    pub fn as_reference(&self) -> Result<&NamedPropRef, PonTranslateErr> {
        match self {
            &Pon::Reference(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Reference".to_string(), found: format!("{:?}", self) })
        }
    }


    // deprecated

    pub fn as_transform(&self) -> Result<&TypedPon, PonTranslateErr> {
        self.translate()
    }
    pub fn as_float(&self) -> Result<&f32, PonTranslateErr> {
        self.translate()
    }
    pub fn as_integer(&self) -> Result<&i64, PonTranslateErr> {
        self.translate()
    }
    pub fn as_string(&self) -> Result<&str, PonTranslateErr> {
        self.translate()
    }
    pub fn as_object(&self) -> Result<&HashMap<String, Pon>, PonTranslateErr> {
        self.translate()
    }
    pub fn get_object_field(&self, field: &str) -> Result<&Pon, PonTranslateErr> {
        self.field(field)
    }
    pub fn as_array(&self) -> Result<&Vec<Pon>, PonTranslateErr> {
        self.translate()
    }
    pub fn as_float_array(&self) -> Result<Cow<Vec<f32>>, PonTranslateErr> {
        self.translate()
    }
    pub fn as_integer_array(&self) -> Result<Cow<Vec<i64>>, PonTranslateErr> {
        self.translate()
    }
}
