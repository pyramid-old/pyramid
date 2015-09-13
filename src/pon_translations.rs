
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
        }
    }
}

pub struct TranslateContext<'a> {
    pub document: Option<&'a Document>,
}

pub trait Translatable<'a, 'b, T> {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<T, PonTranslateErr>;
}

impl<'a, 'b> Translatable<'a, 'b, &'a TypedPon> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<&'a TypedPon, PonTranslateErr> {
        match self {
            &Pon::TypedPon(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "TypedPon".to_string(), found: format!("{:?}", self) })
        }
    }
}

impl<'a, 'b> Translatable<'a, 'b, f32> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<f32, PonTranslateErr> {
        match self {
            &Pon::Float(ref value) => Ok(*value),
            &Pon::Integer(ref value) => Ok(*value as f32),
            _ => Err(PonTranslateErr::MismatchType { expected: "Float".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b> Translatable<'a, 'b, i64> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<i64, PonTranslateErr> {
        match self {
            &Pon::Integer(ref value) => Ok(*value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Integer".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b> Translatable<'a, 'b, &'a str> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<&'a str, PonTranslateErr> {
        match self {
            &Pon::String(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "String".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b> Translatable<'a, 'b, &'a bool> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<&'a bool, PonTranslateErr> {
        match self {
            &Pon::Boolean(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Boolean".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b> Translatable<'a, 'b, Cow<'a, Vec<f32>>> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<Cow<'a, Vec<f32>>, PonTranslateErr> {
        match self {
            &Pon::Array(ref arr) => {
                let mut res_arr = vec![];
                for v in arr {
                    res_arr.push(try!(v.translate::<f32>(context)));
                }
                return Ok(Cow::Owned(res_arr));
            },
            &Pon::FloatArray(ref value) => Ok(Cow::Borrowed(&value)),
            _ => Err(PonTranslateErr::MismatchType { expected: "Array or FloatArray".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b> Translatable<'a, 'b, Cow<'a, Vec<i64>>> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<Cow<'a, Vec<i64>>, PonTranslateErr> {
        match self {
            &Pon::Array(ref arr) => {
                let mut res_arr = vec![];
                for v in arr {
                    res_arr.push(try!(v.translate::<i64>(context)));
                }
                return Ok(Cow::Owned(res_arr));
            },
            &Pon::IntegerArray(ref value) => Ok(Cow::Borrowed(&value)),
            _ => Err(PonTranslateErr::MismatchType { expected: "Array or IntegerArray".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b> Translatable<'a, 'b, &'a Vec<Pon>> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<&'a Vec<Pon>, PonTranslateErr> {
        match self {
            &Pon::Array(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Array".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b> Translatable<'a, 'b, &'a HashMap<String, Pon>> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<&'a HashMap<String, Pon>, PonTranslateErr> {
        match self {
            &Pon::Object(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Object".to_string(), found: format!("{:?}", self) })
        }
    }
}


impl<'a, 'b, T> Translatable<'a, 'b, Vec<T>> for Pon where Pon: Translatable<'a, 'b, T> {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<Vec<T>, PonTranslateErr> {
        let source: &Vec<Pon> = try!(self.translate::<&Vec<Pon>>(context));
        let mut out = vec![];
        for v in source {
            out.push(From::from(try!(v.translate(context))));
        }
        Ok(out)
    }
}


#[test]
fn test_translate_integer() {
    let mut context = TranslateContext { document: None };
    let node = Pon::Integer(5);
    let i: i64 = node.translate(&mut context).unwrap();
    assert_eq!(i, 5);
}
