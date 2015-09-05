use pon::*;
use std::collections::HashMap;

#[test]
fn test_float() {
    let v = Pon::from_string("5.0");
    assert_eq!(v, Ok(Pon::Float(5.0)));
}

#[test]
fn test_neg_float() {
    let v = Pon::from_string("-5.0");
    assert_eq!(v, Ok(Pon::Float(-5.0)));
}

#[test]
fn test_float_empty_space() {
    let v = Pon::from_string(" 5.0 ");
    assert_eq!(v, Ok(Pon::Float(5.0)));
}

#[test]
fn test_integer() {
    let v = Pon::from_string("5");
    assert_eq!(v, Ok(Pon::Integer(5)));
}

#[test]
fn test_string() {
    let v = Pon::from_string("'hi'");
    assert_eq!(v, Ok(Pon::String("hi".to_string())));
}

#[test]
fn test_empty_object() {
    let v = Pon::from_string("{}");
    assert_eq!(v, Ok(Pon::Object(HashMap::new())));
}

#[test]
fn test_object_one() {
    let v = Pon::from_string("{ lol: 5.0 }");
    assert_eq!(v, Ok(Pon::Object(hashmap!{
        "lol".to_string() => Pon::Float(5.0)
    })));
}

#[test]
fn test_object_two() {
    let v = Pon::from_string("{ lol: 5.0, hey: 1.1 }");
    assert_eq!(v, Ok(Pon::Object(hashmap!{
        "lol".to_string() => Pon::Float(5.0),
        "hey".to_string() => Pon::Float(1.1)
    })));
}

#[test]
fn test_object_complex() {
    let v = Pon::from_string("{ a: [0.0, 0.5], b: [0] }");
    assert_eq!(v, Ok(Pon::Object(hashmap!{
        "a".to_string() => Pon::Array(vec![Pon::Float(0.0), Pon::Float(0.5)]),
        "b".to_string() => Pon::Array(vec![Pon::Integer(0)])
    })));
}

#[test]
fn test_array_empty() {
    let v = Pon::from_string("[]");
    assert_eq!(v, Ok(Pon::Array(vec![])));
}

#[test]
fn test_array_one() {
    let v = Pon::from_string("[5.0]");
    assert_eq!(v, Ok(Pon::Array(vec![Pon::Float(5.0)])));
}

#[test]
fn test_array_two() {
    let v = Pon::from_string("[5.0, 3.31]");
    assert_eq!(v, Ok(Pon::Array(vec![Pon::Float(5.0), Pon::Float(3.31)])));
}


#[test]
fn test_transform_nil() {
    let v = Pon::from_string("static_mesh()");
    assert_eq!(v, Ok(Pon::TypedPon(Box::new(TypedPon { type_name: "static_mesh".to_string(), data: Pon::Nil }))));
}

#[test]
fn test_transform_arg() {
    let v = Pon::from_string("static_mesh{ vertices: [0.0, -0.5], indices: [0, 1] }");
    let mut hm = HashMap::new();
    hm.insert("vertices".to_string(), Pon::Array(vec![Pon::Float(0.0), Pon::Float(-0.5)]));
    hm.insert("indices".to_string(),  Pon::Array(vec![Pon::Integer(0), Pon::Integer(1)]));
    assert_eq!(v, Ok(Pon::TypedPon(Box::new(TypedPon { type_name: "static_mesh".to_string(), data: Pon::Object(hm) }))));
}

#[test]
fn test_transform_number() {
    let v = Pon::from_string("static_mesh 5.0");
    assert_eq!(v, Ok(Pon::TypedPon(Box::new(TypedPon { type_name: "static_mesh".to_string(), data: Pon::Float(5.0) }))));
}

#[test]
fn test_dependency_reference() {
    let v = Pon::from_string("@some.test");
    assert_eq!(v, Ok(Pon::DependencyReference(NamedPropRef::new(EntityPath::Named("some".to_string()), "test"))));
}

#[test]
fn test_reference() {
    let v = Pon::from_string("some.test");
    assert_eq!(v, Ok(Pon::Reference(NamedPropRef::new(EntityPath::Named("some".to_string()), "test"))));
}

#[test]
fn test_path() {
    let v = Pon::from_string("some:else.test");
    assert_eq!(v, Ok(Pon::Reference(NamedPropRef::new(EntityPath::Search(Box::new(EntityPath::Named("some".to_string())), "else".to_string()), "test"))));
}

#[test]
fn test_multiline() {
    let v = Pon::from_string("{
        }");
    assert_eq!(v, Ok(Pon::Object(HashMap::new())));
}
