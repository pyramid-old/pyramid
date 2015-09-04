use pon::*;
use cgmath::*;
use std::borrow::Cow;

impl<'a, 'b, C> Translatable<'a, 'b, Vector3<f32>, C> for Pon {
    fn translate(&'a self, context: &'b C) -> Result<Vector3<f32>, PonTranslateErr> {
        let data = match self {
            &Pon::TypedPon(box TypedPon { ref type_name, ref data }) => {
                match type_name.as_str() {
                    "vec3" => data,
                    _ => return Err(PonTranslateErr::UnrecognizedType(type_name.to_string()))
                }
            },
            &Pon::Object(..) => self,
            _ => return Err(PonTranslateErr::MismatchType { expected: "TypedPon or Object".to_string(), found: format!("{:?}", self) })
        };
        let x: f32 = try!(data.field_as_or(context, "x", 0.0));
        let y: f32 = try!(data.field_as_or(context, "y", 0.0));
        let z: f32 = try!(data.field_as_or(context, "z", 0.0));
        Ok(Vector3::new(x, y, z))
    }
}

impl<'a, 'b, C> Translatable<'a, 'b, Matrix4<f32>, C> for Pon {
    fn translate(&'a self, context: &'b C) -> Result<Matrix4<f32>, PonTranslateErr> {
        let &TypedPon { ref type_name, ref data } = from_pon!(self, self, context);
        match type_name.as_str() {
            "matrix" => {
                let data: Cow<'a, Vec<f32>> = from_pon!(self, data, context);
                return Ok(Matrix4::new(
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                    data[8], data[9], data[10], data[11],
                    data[12], data[13], data[14], data[15]));
            },
            "translate" => {
                let vec3: Vector3<f32> = from_pon!(self, data, context);
                return Ok(Matrix4::from_translation(&vec3));
            },
            "rotate_x" => {
                let v: &f32 = from_pon!(self, data, context);
                return Ok(Quaternion::from_angle_x(Rad { s: *v }).into());
            },
            "rotate_y" => {
                let v: &f32 = from_pon!(self, data, context);
                return Ok(Quaternion::from_angle_y(Rad { s: *v }).into());
            },
            "rotate_z" => {
                let v: &f32 = from_pon!(self, data, context);
                return Ok(Quaternion::from_angle_z(Rad { s: *v }).into());
            },
            "scale" => {
                let v: Vector3<f32> = from_pon!(self, data, context);
                let mat = Matrix4::new(
                         v.x,  zero(), zero(), zero(),
                         zero(), v.y,  zero(), zero(),
                         zero(), zero(), v.z,  zero(),
                         zero(), zero(), zero(), one());
                return Ok(mat);
            },
            "lookat" => {
                let eye: Vector3<f32> = try!(data.field_as(context, "eye"));
                let center: Vector3<f32> = try!(data.field_as(context, "center"));
                let up: Vector3<f32> = try!(data.field_as_or(context, "up", Vector3::new(0.0, 0.0, 1.0)));
                return Ok(Matrix4::look_at(&Point3::from_vec(&eye), &Point3::from_vec(&center), &up));
            },
            "projection" => {
                let fovy: f32 = try!(data.field_as_or(context, "fovy", 1.0));
                let aspect: f32 = try!(data.field_as_or(context, "aspect", 1.0));
                let near: f32 = try!(data.field_as_or(context, "near", 0.1));
                let far: f32 = try!(data.field_as_or(context, "far", 10.0));
                return Ok(perspective(Rad { s: fovy }, aspect, near, far));
            },
            "mul" => {
                let data: Vec<Matrix4<f32>> = from_pon!(self, data, context);
                let mut a = Matrix4::identity();
                for b in data {
                    a = a * b;
                }
                return Ok(a);
            },
            _ => Err(PonTranslateErr::UnrecognizedType(type_name.to_string()))
        }
    }
}

#[test]
fn test_pon_to_cgmath() {
    let pon = Pon::from_string("mul [ translate { x: 1.0, y: 1.0, z: 1.0 }, translate vec3 { x: -1.0, y: 0.0, z: -1.0 } ]").unwrap();
    let mat = pon.translate(&());
    assert_eq!(mat, Ok(Matrix4::from_translation(&Vector3::new(0.0, 1.0, 0.0))));
}
