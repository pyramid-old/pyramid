use document::EntityId;

use std::collections::HashMap;
use std::hash::Hasher;
use std::hash::Hash;
use std::cmp::Eq;
use std::borrow::Cow;

macro_rules! translate_pon {
    ($self_:expr, $node:expr, $context:expr) => (match ($node).translate($context) {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            return ::std::result::Result::Err($crate::pon::PonTranslateErr::InnerError {
                pon: $self_.clone(),
                failing_inner_pon: $node.clone(),
                error: ::std::boxed::Box::new(::std::convert::From::from(err))
            });
        }
    })
}

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

pub trait Translatable<'a, 'b, T, C> {
    fn translate(&'a self, context: &'b C) -> Result<T, PonTranslateErr>;
}

impl<'a, 'b, C> Translatable<'a, 'b, &'a TypedPon, C> for Pon {
    fn translate(&'a self, _: &'b C) -> Result<&'a TypedPon, PonTranslateErr> {
        match self {
            &Pon::TypedPon(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "TypedPon".to_string(), found: format!("{:?}", self) })
        }
    }
}

impl<'a, 'b, C> Translatable<'a, 'b, &'a f32, C> for Pon {
    fn translate(&'a self, _: &'b C) -> Result<&'a f32, PonTranslateErr> {
        match self {
            &Pon::Float(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Float".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b, C> Translatable<'a, 'b, &'a i64, C> for Pon {
    fn translate(&'a self, _: &'b C) -> Result<&'a i64, PonTranslateErr> {
        match self {
            &Pon::Integer(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Integer".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b, C> Translatable<'a, 'b, &'a String, C> for Pon {
    fn translate(&'a self, _: &'b C) -> Result<&'a String, PonTranslateErr> {
        match self {
            &Pon::String(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "String".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b, C> Translatable<'a, 'b, Cow<'a, Vec<f32>>, C> for Pon {
    fn translate(&'a self, _: &'b C) -> Result<Cow<'a, Vec<f32>>, PonTranslateErr> {
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
impl<'a, 'b, C> Translatable<'a, 'b, Cow<'a, Vec<i64>>, C> for Pon {
    fn translate(&'a self, _: &'b C) -> Result<Cow<'a, Vec<i64>>, PonTranslateErr> {
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
impl<'a, 'b, C> Translatable<'a, 'b, &'a Vec<Pon>, C> for Pon {
    fn translate(&'a self, _: &'b C) -> Result<&'a Vec<Pon>, PonTranslateErr> {
        match self {
            &Pon::Array(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Array".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b, C> Translatable<'a, 'b, &'a HashMap<String, Pon>, C> for Pon {
    fn translate(&'a self, _: &'b C) -> Result<&'a HashMap<String, Pon>, PonTranslateErr> {
        match self {
            &Pon::Object(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Object".to_string(), found: format!("{:?}", self) })
        }
    }
}

pub struct DependencyReferenceContext;
impl<'a, 'b> Translatable<'a, 'b, &'a NamedPropRef, DependencyReferenceContext> for Pon {
    fn translate(&'a self, _: &'b DependencyReferenceContext) -> Result<&'a NamedPropRef, PonTranslateErr> {
        match self {
            &Pon::DependencyReference(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "DependencyReference".to_string(), found: format!("{:?}", self) })
        }
    }
}

pub struct ReferenceContext;
impl<'a, 'b> Translatable<'a, 'b, &'a NamedPropRef, ReferenceContext> for Pon {
    fn translate(&'a self, _: &'b ReferenceContext) -> Result<&'a NamedPropRef, PonTranslateErr> {
        match self {
            &Pon::Reference(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Reference".to_string(), found: format!("{:?}", self) })
        }
    }
}


pub struct GetFieldContext(String);
impl<'a, 'b> Translatable<'a, 'b, &'a Pon, GetFieldContext> for Pon {
    fn translate(&'a self, context: &'b GetFieldContext) -> Result<&'a Pon, PonTranslateErr> {
        let &GetFieldContext(ref field) = context;
        match self {
            &Pon::Object(ref value) => match value.get(field) {
                Some(value) => Ok(value),
                None => Err(PonTranslateErr::NoSuchField { field: field.to_string() })
            },
            _ => Err(PonTranslateErr::MismatchType { expected: "Object".to_string(), found: format!("{:?}", self) })
        }
    }
}


#[test]
fn test_translate_dependency_reference() {
    let node = Pon::DependencyReference(NamedPropRef { entity_name: "test".to_string(), property_key: "x".to_string() });
    let npr: &NamedPropRef = node.translate(&DependencyReferenceContext).unwrap();
    assert_eq!(*npr, NamedPropRef { entity_name: "test".to_string(), property_key: "x".to_string() });
}

#[test]
fn test_translate_integer() {
    let node = Pon::Integer(5);
    let i: &i64 = node.translate(&()).unwrap();
    assert_eq!(*i, 5);
}

#[test]
fn test_translate_macro() {
    #[derive(PartialEq, Debug)]
    struct TestMacro { x: i64 }
    impl<'a, 'b, C> Translatable<'a, 'b, TestMacro, C> for Pon {
        fn translate(&'a self, context: &'b C) -> Result<TestMacro, PonTranslateErr> {
            let x: &i64 = translate_pon!(self, self, context);
            Ok(TestMacro { x: *x })
        }
    }
    let node = Pon::Integer(5);
    assert_eq!(node.translate(&()), Ok(TestMacro { x: 5}));
}

#[test]
fn test_translate_get_field() {
    let node = Pon::Object(hashmap!( "hello".to_string() => Pon::Integer(5) ));
    let field: &Pon = node.translate(&GetFieldContext("hello".to_string())).unwrap();
    assert_eq!(*field, Pon::Integer(5));
}



#[derive(PartialEq, Debug, Clone)]
pub enum PonTranslateErr {
    MismatchType { expected: String, found: String },
    NoSuchField { field: String },
    UnrecognizedTypedPon(String),
    InnerError { pon: Pon, failing_inner_pon: Pon, error: Box<PonTranslateErr> },
    Generic(String)
}

impl Pon {
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
    pub fn as_transform(&self) -> Result<&TypedPon, PonTranslateErr> {
        self.translate(&())
    }
    pub fn as_float(&self) -> Result<&f32, PonTranslateErr> {
        self.translate(&())
    }
    pub fn as_integer(&self) -> Result<&i64, PonTranslateErr> {
        self.translate(&())
    }
    pub fn as_string(&self) -> Result<&String, PonTranslateErr> {
        self.translate(&())
    }
    pub fn as_reference(&self) -> Result<&NamedPropRef, PonTranslateErr> {
        self.translate(&ReferenceContext)
    }
    pub fn as_object(&self) -> Result<&HashMap<String, Pon>, PonTranslateErr> {
        self.translate(&())
    }
    pub fn get_object_field(&self, field: &str) -> Result<&Pon, PonTranslateErr> {
        self.translate(&GetFieldContext(field.to_string()))
    }
    pub fn as_array(&self) -> Result<&Vec<Pon>, PonTranslateErr> {
        self.translate(&())
    }
    pub fn as_float_array(&self) -> Result<Cow<Vec<f32>>, PonTranslateErr> {
        self.translate(&())
    }
    pub fn as_integer_array(&self) -> Result<Cow<Vec<i64>>, PonTranslateErr> {
        self.translate(&())
    }
}
