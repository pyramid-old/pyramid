
use std::cmp;
use std::borrow::Cow;
use std::collections::HashMap;

use pon::*;
use document::*;

#[derive(PartialEq, Debug, Clone)]
pub enum PonTranslateErr {
    MismatchType { expected: String, found: String },
    NoSuchField { field: String },
    InvalidValue { value: String },
    UnrecognizedType(String),
    ReferenceToNonExistentProperty(NamedPropRef),
    InnerError { in_pon: Pon, error: Box<PonTranslateErr>, trying_to_translate_to: String },
    Generic(String)
}

impl ToString for PonTranslateErr {
    fn to_string(&self) -> String {
        match self {
            &PonTranslateErr::MismatchType { ref expected, ref found  } => format!("Expected {}, found {}", expected, found),
            &PonTranslateErr::NoSuchField { ref field  } => format!("No such field: {}", field),
            &PonTranslateErr::InvalidValue { ref value  } => format!("Invalid value: {}", value),
            &PonTranslateErr::UnrecognizedType(ref value) => format!("Unregcognized type: {}", value),
            &PonTranslateErr::InnerError { ref in_pon, ref error, ref trying_to_translate_to } => {
                let p = in_pon.to_string();
                let p = if p.len() < 50 {
                    p.to_string()
                } else {
                    format!("{}...", &p[0..50])
                };
                format!("while trying to translate {} to {} got error: {}", p, trying_to_translate_to, error.to_string())
            },
            &PonTranslateErr::Generic(ref value) => format!("Generic error: {}", value),
            _ => format!("{:?}", self)
        }
    }
}

pub struct TranslateContext<'a> {
    pub document: Option<&'a Document>,
}
impl<'a> TranslateContext<'a> {
    pub fn empty() -> TranslateContext<'a> {
        TranslateContext {
            document: None
        }
    }
    pub fn from_doc(document: &'a mut Document) -> TranslateContext {
        TranslateContext {
            document: Some(document)
        }
    }
}

pub trait Translatable<T: 'static> {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<T, PonTranslateErr>;
}

impl Translatable<f32> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<f32, PonTranslateErr> {
        match self {
            &Pon::Float(ref value) => Ok(*value),
            &Pon::Integer(ref value) => Ok(*value as f32),
            _ => Err(PonTranslateErr::MismatchType { expected: "Float".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl Translatable<i64> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<i64, PonTranslateErr> {
        match self {
            &Pon::Integer(ref value) => Ok(*value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Integer".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl Translatable<String> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<String, PonTranslateErr> {
        match self {
            &Pon::String(ref value) => Ok(value.to_string()),
            _ => Err(PonTranslateErr::MismatchType { expected: "String".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl Translatable<bool> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<bool, PonTranslateErr> {
        match self {
            &Pon::Boolean(ref value) => Ok(*value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Boolean".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl Translatable<Vec<f32>> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<Vec<f32>, PonTranslateErr> {
        match self {
            &Pon::Array(ref arr) => {
                let mut res_arr = vec![];
                for v in arr {
                    res_arr.push(try!(v.translate::<f32>(context)));
                }
                return Ok(res_arr);
            },
            &Pon::FloatArray(ref value) => Ok(value.clone()),
            _ => Err(PonTranslateErr::MismatchType { expected: "Array or FloatArray".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl Translatable<Vec<i64>> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<Vec<i64>, PonTranslateErr> {
        match self {
            &Pon::Array(ref arr) => {
                let mut res_arr = vec![];
                for v in arr {
                    res_arr.push(try!(v.translate::<i64>(context)));
                }
                return Ok(res_arr);
            },
            &Pon::IntegerArray(ref value) => Ok(value.clone()),
            _ => Err(PonTranslateErr::MismatchType { expected: "Array or IntegerArray".to_string(), found: format!("{:?}", self) })
        }
    }
}

pub struct PonAutoVec<T>(pub Vec<T>);

impl<T: 'static> Translatable<PonAutoVec<T>> for Pon where Pon: Translatable<T> {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<PonAutoVec<T>, PonTranslateErr> {
        self.as_array(|source| {
            let mut out = vec![];
            for v in source {
                out.push(From::from(try!(v.translate(context))));
            }
            Ok(PonAutoVec(out))
        })
    }
}


#[test]
fn test_translate_integer() {
    let node = Pon::Integer(5);
    let i: i64 = node.translate(&mut TranslateContext::empty()).unwrap();
    assert_eq!(i, 5);
}
