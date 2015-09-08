
use std::cmp;
use std::borrow::Cow;
use std::collections::HashMap;

use pon::*;

#[derive(PartialEq, Debug, Clone)]
pub enum PonTranslateErr {
    MismatchType { expected: String, found: String },
    NoSuchField { field: String },
    InvalidValue { value: String },
    UnrecognizedType(String),
    InnerError { in_pon: Pon, error: Box<PonTranslateErr> },
    Generic(String)
}

impl ToString for PonTranslateErr {
    fn to_string(&self) -> String {
        match self {
            &PonTranslateErr::MismatchType { ref expected, ref found  } => format!("Expected {}, found {}", expected, found),
            &PonTranslateErr::NoSuchField { ref field  } => format!("No such field: {}", field),
            &PonTranslateErr::InvalidValue { ref value  } => format!("Invalid value: {}", value),
            &PonTranslateErr::UnrecognizedType(ref value) => format!("Unregcognized type: {}", value),
            &PonTranslateErr::InnerError { ref in_pon, ref error } => {
                let p = in_pon.to_string();
                format!("{} in {}...", error.to_string(), &p[0..cmp::min(50, p.len())])
            },
            &PonTranslateErr::Generic(ref value) => format!("Generic error: {}", value),
        }
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
impl<'a> Translatable<'a, &'a bool> for Pon {
    fn inner_translate(&'a self) -> Result<&'a bool, PonTranslateErr> {
        match self {
            &Pon::Boolean(ref value) => Ok(&value),
            _ => Err(PonTranslateErr::MismatchType { expected: "Boolean".to_string(), found: format!("{:?}", self) })
        }
    }
}
impl<'a> Translatable<'a, Cow<'a, Vec<f32>>> for Pon {
    fn inner_translate(&'a self) -> Result<Cow<'a, Vec<f32>>, PonTranslateErr> {
        match self {
            &Pon::Array(ref arr) => {
                let mut res_arr = vec![];
                for v in arr {
                    res_arr.push(try!(v.translate::<f32>()));
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
                    res_arr.push(try!(v.translate::<i64>()));
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
