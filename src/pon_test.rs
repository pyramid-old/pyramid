peg_file! pon_parse("pon.rustpeg");
use pon::*;
use std::collections::HashMap;

#[test]
fn test_float() {
    let v = pon_parse::body("5.0");
    assert_eq!(v, Ok(Pon::Float(5.0)));
}

#[test]
fn test_neg_float() {
    let v = pon_parse::body("-5.0");
    assert_eq!(v, Ok(Pon::Float(-5.0)));
}

#[test]
fn test_float_empty_space() {
    let v = pon_parse::body(" 5.0 ");
    assert_eq!(v, Ok(Pon::Float(5.0)));
}

#[test]
fn test_integer() {
    let v = pon_parse::body("5");
    assert_eq!(v, Ok(Pon::Integer(5)));
}

#[test]
fn test_string() {
    let v = pon_parse::body("'hi'");
    assert_eq!(v, Ok(Pon::String("hi".to_string())));
}

#[test]
fn test_empty_object() {
    let v = pon_parse::body("{}");
    assert_eq!(v, Ok(Pon::Object(HashMap::new())));
}

#[test]
fn test_object_one() {
    let v = pon_parse::body("{ lol: 5.0 }");
    assert_eq!(v, Ok(Pon::Object(hashmap!{
        "lol".to_string() => Pon::Float(5.0)
    })));
}

#[test]
fn test_object_two() {
    let v = pon_parse::body("{ lol: 5.0, hey: 1.1 }");
    assert_eq!(v, Ok(Pon::Object(hashmap!{
        "lol".to_string() => Pon::Float(5.0),
        "hey".to_string() => Pon::Float(1.1)
    })));
}

#[test]
fn test_object_complex() {
    let v = pon_parse::body("{ a: [0.0, 0.5], b: [0] }");
    assert_eq!(v, Ok(Pon::Object(hashmap!{
        "a".to_string() => Pon::Array(vec![Pon::Float(0.0), Pon::Float(0.5)]),
        "b".to_string() => Pon::Array(vec![Pon::Integer(0)])
    })));
}

#[test]
fn test_array_empty() {
    let v = pon_parse::body("[]");
    assert_eq!(v, Ok(Pon::Array(vec![])));
}

#[test]
fn test_array_one() {
    let v = pon_parse::body("[5.0]");
    assert_eq!(v, Ok(Pon::Array(vec![Pon::Float(5.0)])));
}

#[test]
fn test_array_two() {
    let v = pon_parse::body("[5.0, 3.31]");
    assert_eq!(v, Ok(Pon::Array(vec![Pon::Float(5.0), Pon::Float(3.31)])));
}


#[test]
fn test_transform_nil() {
    let v = pon_parse::body("static_mesh()");
    assert_eq!(v, Ok(Pon::PropTransform(Box::new(PropTransform { name: "static_mesh".to_string(), arg: Pon::Nil }))));
}

#[test]
fn test_transform_arg() {
    let v = pon_parse::body("static_mesh{ vertices: [0.0, -0.5], indices: [0, 1] }");
    let mut hm = HashMap::new();
    hm.insert("vertices".to_string(), Pon::Array(vec![Pon::Float(0.0), Pon::Float(-0.5)]));
    hm.insert("indices".to_string(),  Pon::Array(vec![Pon::Integer(0), Pon::Integer(1)]));
    assert_eq!(v, Ok(Pon::PropTransform(Box::new(PropTransform { name: "static_mesh".to_string(), arg: Pon::Object(hm) }))));
}

#[test]
fn test_transform_number() {
    let v = pon_parse::body("static_mesh 5.0");
    assert_eq!(v, Ok(Pon::PropTransform(Box::new(PropTransform { name: "static_mesh".to_string(), arg: Pon::Float(5.0) }))));
}

#[test]
fn test_dependency_reference() {
    let v = pon_parse::body("@some.test");
    assert_eq!(v, Ok(Pon::DependencyReference(NamedPropRef { entity_name: "some".to_string(), property_key: "test".to_string() })));
}

#[test]
fn test_reference() {
    let v = pon_parse::body("some.test");
    assert_eq!(v, Ok(Pon::Reference(NamedPropRef { entity_name: "some".to_string(), property_key: "test".to_string() })));
}

#[test]
fn test_multiline() {
    let v = pon_parse::body("{
        }");
    assert_eq!(v, Ok(Pon::Object(HashMap::new())));
}
