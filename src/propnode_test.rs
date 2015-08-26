peg_file! propnode_parse("propnode.rustpeg");
use propnode::*;
use std::collections::HashMap;

#[test]
fn test_float() {
    let v = propnode_parse::body("5.0");
    assert_eq!(v, Ok(PropNode::Float(5.0)));
}

#[test]
fn test_neg_float() {
    let v = propnode_parse::body("-5.0");
    assert_eq!(v, Ok(PropNode::Float(-5.0)));
}

#[test]
fn test_float_empty_space() {
    let v = propnode_parse::body(" 5.0 ");
    assert_eq!(v, Ok(PropNode::Float(5.0)));
}

#[test]
fn test_integer() {
    let v = propnode_parse::body("5");
    assert_eq!(v, Ok(PropNode::Integer(5)));
}

#[test]
fn test_string() {
    let v = propnode_parse::body("'hi'");
    assert_eq!(v, Ok(PropNode::String("hi".to_string())));
}

#[test]
fn test_empty_object() {
    let v = propnode_parse::body("{}");
    assert_eq!(v, Ok(PropNode::Object(HashMap::new())));
}

#[test]
fn test_object_one() {
    let v = propnode_parse::body("{ lol: 5.0 }");
    assert_eq!(v, Ok(PropNode::Object(hashmap!{
        "lol".to_string() => PropNode::Float(5.0)
    })));
}

#[test]
fn test_object_two() {
    let v = propnode_parse::body("{ lol: 5.0, hey: 1.1 }");
    assert_eq!(v, Ok(PropNode::Object(hashmap!{
        "lol".to_string() => PropNode::Float(5.0),
        "hey".to_string() => PropNode::Float(1.1)
    })));
}

#[test]
fn test_object_complex() {
    let v = propnode_parse::body("{ a: [0.0, 0.5], b: [0] }");
    assert_eq!(v, Ok(PropNode::Object(hashmap!{
        "a".to_string() => PropNode::Array(vec![PropNode::Float(0.0), PropNode::Float(0.5)]),
        "b".to_string() => PropNode::Array(vec![PropNode::Integer(0)])
    })));
}

#[test]
fn test_array_empty() {
    let v = propnode_parse::body("[]");
    assert_eq!(v, Ok(PropNode::Array(vec![])));
}

#[test]
fn test_array_one() {
    let v = propnode_parse::body("[5.0]");
    assert_eq!(v, Ok(PropNode::Array(vec![PropNode::Float(5.0)])));
}

#[test]
fn test_array_two() {
    let v = propnode_parse::body("[5.0, 3.31]");
    assert_eq!(v, Ok(PropNode::Array(vec![PropNode::Float(5.0), PropNode::Float(3.31)])));
}


#[test]
fn test_transform_nil() {
    let v = propnode_parse::body("static_mesh()");
    assert_eq!(v, Ok(PropNode::PropTransform(Box::new(PropTransform { name: "static_mesh".to_string(), arg: PropNode::Nil }))));
}

#[test]
fn test_transform_arg() {
    let v = propnode_parse::body("static_mesh{ vertices: [0.0, -0.5], indices: [0, 1] }");
    let mut hm = HashMap::new();
    hm.insert("vertices".to_string(), PropNode::Array(vec![PropNode::Float(0.0), PropNode::Float(-0.5)]));
    hm.insert("indices".to_string(),  PropNode::Array(vec![PropNode::Integer(0), PropNode::Integer(1)]));
    assert_eq!(v, Ok(PropNode::PropTransform(Box::new(PropTransform { name: "static_mesh".to_string(), arg: PropNode::Object(hm) }))));
}

#[test]
fn test_transform_number() {
    let v = propnode_parse::body("static_mesh 5.0");
    assert_eq!(v, Ok(PropNode::PropTransform(Box::new(PropTransform { name: "static_mesh".to_string(), arg: PropNode::Float(5.0) }))));
}

#[test]
fn test_dependency_reference() {
    let v = propnode_parse::body("@some.test");
    assert_eq!(v, Ok(PropNode::DependencyReference(NamedPropRef { entity_name: "some".to_string(), property_key: "test".to_string() })));
}

#[test]
fn test_reference() {
    let v = propnode_parse::body("some.test");
    assert_eq!(v, Ok(PropNode::Reference(NamedPropRef { entity_name: "some".to_string(), property_key: "test".to_string() })));
}

#[test]
fn test_multiline() {
    let v = propnode_parse::body("{
        }");
    assert_eq!(v, Ok(PropNode::Object(HashMap::new())));
}
