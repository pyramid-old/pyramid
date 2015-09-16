use pon::*;
use cgmath::*;
use std::borrow::Cow;
use document::*;

impl Translatable<Vector3<f32>> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<Vector3<f32>, PonTranslateErr> {
        match self {
            &Pon::TypedPon(box TypedPon { ref type_name, ref data }) => {
                match type_name.as_str() {
                    "vec3" => {
                        let x: f32 = try!(data.field_as_or("x", 0.0, context));
                        let y: f32 = try!(data.field_as_or("y", 0.0, context));
                        let z: f32 = try!(data.field_as_or("z", 0.0, context));
                        Ok(Vector3::new(x, y, z))
                    },
                    _ => return Err(PonTranslateErr::UnrecognizedType(type_name.to_string()))
                }
            },
            &Pon::Object(..) => {
                let x: f32 = try!(self.field_as_or("x", 0.0, context));
                let y: f32 = try!(self.field_as_or("y", 0.0, context));
                let z: f32 = try!(self.field_as_or("z", 0.0, context));
                Ok(Vector3::new(x, y, z))
            },
            &Pon::Vector3(ref vec3) => Ok(vec3.clone()),
            _ => return Err(PonTranslateErr::MismatchType { expected: "TypedPon or Object".to_string(), found: format!("{:?}", self) })
        }
    }
}

impl ToPon for Vector3<f32> {
    fn to_pon(&self) -> Pon {
        Pon::TypedPon(Box::new(TypedPon {
            type_name: "vec3".to_string(),
            data: Pon::Object(hashmap!(
                "x" => Pon::Float(self.x),
                "y" => Pon::Float(self.y),
                "z" => Pon::Float(self.z)
            ))
        }))
    }
}

#[test]
fn test_vec3_to_pon() {
    let v = Vector3::new(1.0, 2.0, 3.0);
    assert_eq!(v.to_pon(), Pon::from_string("vec3 { x: 1.0, y: 2.0, z: 3.0 }").unwrap());
}

#[test]
fn test_vec3_wrapped() {
    let pon = Pon::Vector3(Vector3::new(1.0, 2.0, 3.0));
    let vec3: Vector3<f32> = pon.translate(&mut TranslateContext::empty()).unwrap();
    assert_eq!(vec3, Vector3::new(1.0, 2.0, 3.0));
}


impl Translatable<Vector4<f32>> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<Vector4<f32>, PonTranslateErr> {
        match self {
            &Pon::TypedPon(box TypedPon { ref type_name, ref data }) => {
                match type_name.as_str() {
                    "vec4" => {
                        let x: f32 = try!(data.field_as_or("x", 0.0, context));
                        let y: f32 = try!(data.field_as_or("y", 0.0, context));
                        let z: f32 = try!(data.field_as_or("z", 0.0, context));
                        let w: f32 = try!(data.field_as_or("w", 0.0, context));
                        Ok(Vector4::new(x, y, z, w))
                    },
                    _ => return Err(PonTranslateErr::UnrecognizedType(type_name.to_string()))
                }
            },
            &Pon::Object(..) => {
                let x: f32 = try!(self.field_as_or("x", 0.0, context));
                let y: f32 = try!(self.field_as_or("y", 0.0, context));
                let z: f32 = try!(self.field_as_or("z", 0.0, context));
                let w: f32 = try!(self.field_as_or("w", 0.0, context));
                Ok(Vector4::new(x, y, z, w))
            },
            &Pon::Vector4(ref vec4) => Ok(vec4.clone()),
            _ => return Err(PonTranslateErr::MismatchType { expected: "TypedPon or Object".to_string(), found: format!("{:?}", self) })
        }
    }
}

impl ToPon for Vector4<f32> {
    fn to_pon(&self) -> Pon {
        Pon::TypedPon(Box::new(TypedPon {
            type_name: "vec4".to_string(),
            data: Pon::Object(hashmap!(
                "x" => Pon::Float(self.x),
                "y" => Pon::Float(self.y),
                "z" => Pon::Float(self.z),
                "w" => Pon::Float(self.w)
            ))
        }))
    }
}

#[test]
fn test_vec4_to_pon() {
    let v = Vector4::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(v.to_pon(), Pon::from_string("vec4 { x: 1.0, y: 2.0, z: 3.0, w: 4.0 }").unwrap());
}

#[test]
fn test_vec4_wrapped() {
    let pon = Pon::Vector4(Vector4::new(1.0, 2.0, 3.0, 4.0));
    let vec4: Vector4<f32> = pon.translate(&mut TranslateContext::empty()).unwrap();
    assert_eq!(vec4, Vector4::new(1.0, 2.0, 3.0, 4.0));
}



impl Translatable<Matrix4<f32>> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<Matrix4<f32>, PonTranslateErr> {
        if let &Pon::Matrix4(ref mat) = self {
            return Ok(mat.clone());
        }
        self.as_typed(|&TypedPon { ref type_name, ref data }| {
            match type_name.as_str() {
                "matrix" => {
                    let data: Vec<f32> = try!(data.translate(context));
                    return Ok(Matrix4::new(
                        data[0], data[1], data[2], data[3],
                        data[4], data[5], data[6], data[7],
                        data[8], data[9], data[10], data[11],
                        data[12], data[13], data[14], data[15]));
                },
                "translate" => {
                    let vec3: Vector3<f32> = try!(data.translate(context));
                    return Ok(Matrix4::from_translation(&vec3));
                },
                "rotate_x" => {
                    let v: f32 = try!(data.translate(context));
                    return Ok(Quaternion::from_angle_x(Rad { s: v }).into());
                },
                "rotate_y" => {
                    let v: f32 = try!(data.translate(context));
                    return Ok(Quaternion::from_angle_y(Rad { s: v }).into());
                },
                "rotate_z" => {
                    let v: f32 = try!(data.translate(context));
                    return Ok(Quaternion::from_angle_z(Rad { s: v }).into());
                },
                "rotate_quaternion" => {
                    let v: Vector4<f32> = try!(data.translate(context));
                    return Ok(Quaternion::new(v.w, v.x, v.y, v.z).into());
                },
                "scale" => {
                    let v: Vector3<f32> = try!(data.translate(context));
                    let mat = Matrix4::new(
                             v.x,  zero(), zero(), zero(),
                             zero(), v.y,  zero(), zero(),
                             zero(), zero(), v.z,  zero(),
                             zero(), zero(), zero(), one());
                    return Ok(mat);
                },
                "lookat" => {
                    let eye: Vector3<f32> = try!(data.field_as("eye", context));
                    let center: Vector3<f32> = try!(data.field_as("center", context));
                    let up: Vector3<f32> = try!(data.field_as_or("up", Vector3::new(0.0, 0.0, 1.0), context));
                    return Ok(Matrix4::look_at(&Point3::from_vec(&eye), &Point3::from_vec(&center), &up));
                },
                "projection" => {
                    let fovy: f32 = try!(data.field_as_or("fovy", 1.0, context));
                    let aspect: f32 = try!(data.field_as_or("aspect", 1.0, context));
                    let near: f32 = try!(data.field_as_or("near", 0.1, context));
                    let far: f32 = try!(data.field_as_or("far", 10.0, context));
                    return Ok(perspective(Rad { s: fovy }, aspect, near, far));
                },
                "mul" => {
                    let data: PonAutoVec<Matrix4<f32>> = try!(data.translate(context));
                    let mut a = Matrix4::identity();
                    for b in data.0 {
                        a = a * b;
                    }
                    return Ok(a);
                },
                _ => Err(PonTranslateErr::UnrecognizedType(type_name.to_string()))
            }
        })
    }
}
impl ToPon for Matrix4<f32> {
    fn to_pon(&self) -> Pon {
        Pon::TypedPon(Box::new(TypedPon {
            type_name: "matrix".to_string(),
            data: Pon::FloatArray(vec![
                self.x.x, self.x.y, self.x.z, self.x.w,
                self.y.x, self.y.y, self.y.z, self.y.w,
                self.z.x, self.z.y, self.z.z, self.z.w,
                self.w.x, self.w.y, self.w.z, self.w.w,
            ])
        }))
    }
}


#[test]
fn test_pon_to_cgmath() {
    let pon = Pon::from_string("mul [ translate { x: 1.0, y: 1.0, z: 1.0 }, translate vec3 { x: -1.0, y: 0.0, z: -1.0 } ]").unwrap();
    let mat = pon.translate(&mut TranslateContext::empty());
    assert_eq!(mat, Ok(Matrix4::from_translation(&Vector3::new(0.0, 1.0, 0.0))));
}
