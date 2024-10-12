pub mod texture;

use std::{
    f32::consts::{FRAC_2_PI, FRAC_PI_2, PI, TAU},
    os::raw::c_float,
    time::Instant,
};

use approx::assert_relative_eq;
use classicube_sys::{
    Camera, Gfx, Gfx_LoadMatrix, Gfx_SetAlphaBlending, Gfx_SetAlphaTest, Gfx_SetFaceCulling,
    Gfx_SetFog, Gfx_SetTexturing, Matrix, MatrixType__MATRIX_VIEW, Matrix_Identity, OwnedTexture,
    Vec3, Vec4, MATH_DEG2RAD, MATH_RAD2DEG,
};
use nalgebra::{
    center, distance, AbstractRotation, Isometry3, IsometryMatrix3, Matrix3, Matrix4, Point3,
    Rotation3, Scale3, Unit, UnitQuaternion, UnitVector3, Vector3,
};
use nalgebra_glm::{
    identity, look_at, quat_look_at, rotate, rotate_x, rotate_y, rotate_z, scale, translate,
};
use texture::create_texture;
use tracing::debug;

use super::{context::vertex_buffer::Texture_Render, render_hook::renderable::Renderable};

pub fn vec3_to_point3(v: &Vec3) -> Point3<f32> {
    Point3::new(v.X, v.Y, v.Z)
}

pub fn vec3_to_vector3(v: &Vec3) -> Vector3<f32> {
    Vector3::new(v.X, v.Y, v.Z)
}

pub struct Laser {
    start_pos: Vec3,
    end_pos: Vec3,
    texture: OwnedTexture,
}

thread_local!(
    static START: Instant = Instant::now();
);

impl Laser {
    pub fn new(start_pos: Vec3, end_pos: Vec3) -> Self {
        let block_width = (end_pos - start_pos).length_squared().sqrt();
        let texture = create_texture(block_width);

        Self {
            start_pos,
            end_pos,
            texture,
        }
    }

    fn render_inner(&mut self) {
        let start_pos = vec3_to_point3(&self.start_pos);
        let end_pos = vec3_to_point3(&self.end_pos);
        let block_width = distance(&start_pos, &end_pos);

        let eye_pos = vec3_to_point3(&unsafe {
            let camera = &*Camera.Active;
            let get_position = camera.GetPosition.unwrap();
            get_position(0.0)
        });

        let height = self.texture.as_texture().Height as f32;
        let width = self.texture.as_texture().Width as f32;

        let forward = (end_pos - start_pos).normalize();
        let eye_dir = (eye_pos - center(&start_pos, &end_pos)).normalize();

        let t = START.with(|s| s.elapsed()).as_secs_f32();

        let right = forward.cross(&Vector3::y_axis());
        let up = right.cross(&forward);
        let rotation =
            Rotation3::from_matrix_unchecked(Matrix3::from_columns(&[right, up, forward]));

        let rotation = rotation
            * Rotation3::from_euler_angles(-90.0f32.to_radians(), -90.0f32.to_radians(), 0.0);

        let mut transform = identity();
        transform = translate(&transform, &start_pos.coords);
        transform *= rotation.to_homogeneous();
        transform = scale(
            &transform,
            &Vector3::new((0.5 * block_width * 2.0) / width, 0.5 / height, 1.0),
        );

        let view = to_na_matrix(unsafe { Gfx.View });
        let m = to_cc_matrix(view * transform);
        unsafe {
            Gfx_LoadMatrix(MatrixType__MATRIX_VIEW, &m);

            Gfx_SetAlphaTest(1);
            Gfx_SetTexturing(1);
            Gfx_SetFaceCulling(1);
            // Gfx_SetFog(0);
            // Gfx_SetAlphaBlending(1);

            Texture_Render(self.texture.as_texture_mut(), true);

            // Gfx_SetAlphaBlending(0);
            Gfx_SetFaceCulling(0);

            Gfx_LoadMatrix(MatrixType__MATRIX_VIEW, &Gfx.View);
        }
    }
}

fn to_cc_matrix<T: Into<Matrix4<f32>>>(na: T) -> Matrix {
    let na = na.into();
    Matrix {
        row1: Vec4 {
            X: na[(0, 0)],
            Y: na[(1, 0)],
            Z: na[(2, 0)],
            W: na[(3, 0)],
        },
        row2: Vec4 {
            X: na[(0, 1)],
            Y: na[(1, 1)],
            Z: na[(2, 1)],
            W: na[(3, 1)],
        },
        row3: Vec4 {
            X: na[(0, 2)],
            Y: na[(1, 2)],
            Z: na[(2, 2)],
            W: na[(3, 2)],
        },
        row4: Vec4 {
            X: na[(0, 3)],
            Y: na[(1, 3)],
            Z: na[(2, 3)],
            W: na[(3, 3)],
        },
    }
}

fn to_na_matrix(cc: Matrix) -> Matrix4<f32> {
    Matrix4::new(
        cc.row1.X, cc.row2.X, cc.row3.X, cc.row4.X, //
        cc.row1.Y, cc.row2.Y, cc.row3.Y, cc.row4.Y, //
        cc.row1.Z, cc.row2.Z, cc.row3.Z, cc.row4.Z, //
        cc.row1.W, cc.row2.W, cc.row3.W, cc.row4.W, //
    )
}

#[test]
fn test_to_matrix() {
    let a = Matrix4::new(
        0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
    );
    assert_eq!(a, a);
    let b = to_cc_matrix(a);
    assert_eq!(b.row4.X, *a.index((0, 3)));
    assert_eq!(b.row4.Y, *a.index((1, 3)));
    assert_eq!(b.row4.Z, *a.index((2, 3)));
    assert_eq!(b.row4.W, *a.index((3, 3)));
    let c = to_na_matrix(b);
    assert_eq!(a, c);
}

impl Renderable for Laser {
    fn render(&mut self) {
        self.render_inner();
    }
}

#[test]
fn test_math1() {
    for (i, (start_pos, end_pos, distance_solution, dir_solution, pitch_solution, yaw_solution)) in
        [
            (
                (64.0, 40.0, 64.0),
                (64.0, 40.0, 50.0),
                14.0,
                (0.0, 0.0, -1.0),
                0.0,
                0.0,
            ),
            // (
            //     (64.0, 40.0, 64.0),
            //     (70.0, 40.0, 64.0),
            //     6.0,
            //     (1.0, 0.0, 0.0),
            //     0.0,
            //     -90.0,
            // ),
            // (
            //     (64.0, 40.0, 64.0),
            //     (64.0, 50.0, 64.0),
            //     10.0,
            //     (0.0, 1.0, 0.0),
            //     90.0,
            //     -180.0,
            // ),
            // (
            //     (64.0, 40.0, 64.0),
            //     (74.0, 40.0, 54.0),
            //     14.142136,
            //     (0.70710677, 0.0, -0.70710677),
            //     0.0,
            //     -45.0,
            // ),
            // (
            //     (64.0, 40.0, 64.0),
            //     (64.0, 40.0, 70.0),
            //     6.0,
            //     (0.0, 0.0, 1.0),
            //     0.0,
            //     -180.0,
            // ),
        ]
        .into_iter()
        .enumerate()
    {
        let start_pos = Point3::<f32>::new(start_pos.0, start_pos.1, start_pos.2);
        let end_pos = Point3::<f32>::new(end_pos.0, end_pos.1, end_pos.2);
        let dir_solution = Vector3::new(dir_solution.0, dir_solution.1, dir_solution.2);

        let block_width = distance(&start_pos, &end_pos);
        assert_eq!(block_width, distance_solution, "iter {i}");

        let dir = (end_pos - start_pos).normalize();
        assert_eq!(dir, dir_solution, "iter {i}");

        let pitch = dir.y.asin();
        let yaw = -dir.x.atan2(-dir.z);

        assert_eq!(pitch.to_degrees(), pitch_solution, "iter {i}");
        assert_eq!(yaw.to_degrees(), yaw_solution, "iter {i}");

        let iso = Rotation3::look_at_rh(&(end_pos - start_pos), &Vector3::y_axis());
        println!("{:#?}", iso);
    }
}

#[test]
fn test_math3() {
    #[allow(clippy::single_element_loop)]
    for (start_pos, end_pos, dir_solution, pitch_solution, yaw_solution) in [
        // (
        //     (64.0f32, 40.0, 64.0),
        //     (64.0f32, 40.0, 50.0),
        //     (0.0, 0.0, -1.0),
        //     0.0,
        //     0.0,
        // ),
        // (
        //     (64.0f32, 40.0, 64.0),
        //     (70.0f32, 40.0, 64.0),
        //     (1.0, 0.0, 0.0),
        //     0.0,
        //     -90.0,
        // ),
        (
            (0.0f32, 0.0, 0.0),
            (0.0f32, 1.0, -1.0),
            (1.0, 0.0, 0.0),
            45.0,
            0.0,
        ),
    ] {
        let start_pos = Point3::new(start_pos.0, start_pos.1, start_pos.2);
        let end_pos = Point3::new(end_pos.0, end_pos.1, end_pos.2);

        // let rot_solution = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
        // println!("{:#?}", rot_solution.to_rotation_matrix());
        // let rot = look_at(&start_pos, &end_pos, &Vector3::y_axis());
        // let rot = rot.remove_row(3);
        // let rot = rot.remove_column(3);
        // println!("{:#?}", rot);

        // let rot = Rotation3::<f32>::new(end_pos - start_pos)
        //     .euler_angles()
        //     .0
        //     .to_degrees();

        let direction = (end_pos - start_pos).normalize();
        let rotation = Rotation3::rotation_between(&-Vector3::z_axis(), &direction).unwrap();

        println!(
            "{} {} {}",
            rotation.euler_angles().0.to_degrees(),
            rotation.euler_angles().1.to_degrees(),
            rotation.euler_angles().2.to_degrees()
        );
    }
}

#[test]
fn test_math2() {
    for (pitch, yaw, roll) in [
        //
        (0.0f32, 0.0f32, 0.0f32),
        (90.0, 0.0, 0.0),
        (0.0, 90.0, 0.0),
        (0.0, 0.0, 90.0),
        (90.0, 90.0, 0.0),
        (90.0, 0.0, 90.0),
        (45.0, 45.0, 0.0),
    ] {
        println!("{pitch} {yaw} {roll}");
        let (pitch, yaw, roll) = (pitch.to_radians(), yaw.to_radians(), roll.to_radians());

        let na = Rotation3::from_euler_angles(pitch, yaw, roll);
        println!("{:#?}", na);

        let cc = Matrix_Mul(
            Matrix_Mul(
                //
                Matrix_RotateX(pitch),
                Matrix_RotateZ(roll),
            ),
            Matrix_RotateY(yaw),
        );
        println!("{:#?}", cc);

        assert_relative_eq!(na[(0, 0)], cc.row1.X);
        assert_relative_eq!(na[(1, 0)], cc.row1.Y);
        assert_relative_eq!(na[(2, 0)], cc.row1.Z);
        assert_relative_eq!(na[(0, 1)], cc.row2.X);
        assert_relative_eq!(na[(1, 1)], cc.row2.Y);
        assert_relative_eq!(na[(2, 1)], cc.row2.Z);
        assert_relative_eq!(na[(0, 2)], cc.row3.X);
        assert_relative_eq!(na[(1, 2)], cc.row3.Y);
        assert_relative_eq!(na[(2, 2)], cc.row3.Z);
    }
}

#[test]
fn test_math4() {
    let cc = Matrix_Translate(1.0, 2.0, 3.0);
    println!("{:#?}", to_na_matrix(cc));

    let na = IsometryMatrix3::new(Vector3::new(1.0, 2.0, 3.0), Vector3::zeros());
    println!("{:#?}", na.to_matrix());

    assert_eq!(to_na_matrix(cc), na.to_matrix());
}

#[test]
fn test_math4_2() {
    let cc = Matrix_Mul(
        Matrix_RotateX(45.0f32.to_radians()),
        Matrix_Translate(1.0, 2.0, 3.0),
    );
    println!("{:#?}", to_na_matrix(cc));

    let na = IsometryMatrix3::new(
        Vector3::new(1.0, 2.0, 3.0),
        Rotation3::from_euler_angles(45.0f32.to_radians(), 0.0, 0.0).scaled_axis(),
    );
    println!("{:#?}", na.to_matrix());

    assert_relative_eq!(to_na_matrix(cc), na.to_matrix());
}

#[test]
fn test_math4_3() {
    let cc = Matrix_Mul(
        Matrix_Scale(2.0, 1.0, 3.0),
        Matrix_Mul(
            Matrix_RotateX(45.0f32.to_radians()),
            Matrix_Translate(1.0, 2.0, 3.0),
        ),
    );
    println!("{:#?}", to_na_matrix(cc));

    let na = scale(
        &IsometryMatrix3::new(
            Vector3::new(1.0, 2.0, 3.0),
            Rotation3::from_euler_angles(45.0f32.to_radians(), 0.0, 0.0).scaled_axis(),
        )
        .to_matrix(),
        &Vector3::new(2.0, 1.0, 3.0),
    );
    println!("{:#?}", na);

    assert_relative_eq!(to_na_matrix(cc), na);
}

fn Matrix_RotateX(angle: f32) -> Matrix {
    let cosA = angle.cos();
    let sinA = angle.sin();
    let mut result = Matrix_Identity;

    result.row2.Y = cosA;
    result.row2.Z = sinA;
    result.row3.Y = -sinA;
    result.row3.Z = cosA;

    result
}

fn Matrix_RotateY(angle: f32) -> Matrix {
    let cosA = angle.cos();
    let sinA = angle.sin();
    let mut result = Matrix_Identity;

    result.row1.X = cosA;
    result.row1.Z = -sinA;
    result.row3.X = sinA;
    result.row3.Z = cosA;
    result
}

fn Matrix_RotateZ(angle: f32) -> Matrix {
    let cosA = angle.cos();
    let sinA = angle.sin();
    let mut result = Matrix_Identity;

    result.row1.X = cosA;
    result.row1.Y = sinA;
    result.row2.X = -sinA;
    result.row2.Y = cosA;
    result
}

fn Matrix_Translate(x: f32, y: f32, z: f32) -> Matrix {
    let mut result = Matrix_Identity;
    result.row4.X = x;
    result.row4.Y = y;
    result.row4.Z = z;
    result
}

fn Matrix_Scale(x: f32, y: f32, z: f32) -> Matrix {
    let mut result = Matrix_Identity;
    result.row1.X = x;
    result.row2.Y = y;
    result.row3.Z = z;
    result
}

fn Matrix_Mul(left: Matrix, right: Matrix) -> Matrix {
    let lM11 = left.row1.X;
    let lM12 = left.row1.Y;
    let lM13 = left.row1.Z;
    let lM14 = left.row1.W;
    let lM21 = left.row2.X;
    let lM22 = left.row2.Y;
    let lM23 = left.row2.Z;
    let lM24 = left.row2.W;
    let lM31 = left.row3.X;
    let lM32 = left.row3.Y;
    let lM33 = left.row3.Z;
    let lM34 = left.row3.W;
    let lM41 = left.row4.X;
    let lM42 = left.row4.Y;
    let lM43 = left.row4.Z;
    let lM44 = left.row4.W;

    let rM11 = right.row1.X;
    let rM12 = right.row1.Y;
    let rM13 = right.row1.Z;
    let rM14 = right.row1.W;
    let rM21 = right.row2.X;
    let rM22 = right.row2.Y;
    let rM23 = right.row2.Z;
    let rM24 = right.row2.W;
    let rM31 = right.row3.X;
    let rM32 = right.row3.Y;
    let rM33 = right.row3.Z;
    let rM34 = right.row3.W;
    let rM41 = right.row4.X;
    let rM42 = right.row4.Y;
    let rM43 = right.row4.Z;
    let rM44 = right.row4.W;

    let mut result = Matrix_Identity;
    result.row1.X = (((lM11 * rM11) + (lM12 * rM21)) + (lM13 * rM31)) + (lM14 * rM41);
    result.row1.Y = (((lM11 * rM12) + (lM12 * rM22)) + (lM13 * rM32)) + (lM14 * rM42);
    result.row1.Z = (((lM11 * rM13) + (lM12 * rM23)) + (lM13 * rM33)) + (lM14 * rM43);
    result.row1.W = (((lM11 * rM14) + (lM12 * rM24)) + (lM13 * rM34)) + (lM14 * rM44);

    result.row2.X = (((lM21 * rM11) + (lM22 * rM21)) + (lM23 * rM31)) + (lM24 * rM41);
    result.row2.Y = (((lM21 * rM12) + (lM22 * rM22)) + (lM23 * rM32)) + (lM24 * rM42);
    result.row2.Z = (((lM21 * rM13) + (lM22 * rM23)) + (lM23 * rM33)) + (lM24 * rM43);
    result.row2.W = (((lM21 * rM14) + (lM22 * rM24)) + (lM23 * rM34)) + (lM24 * rM44);

    result.row3.X = (((lM31 * rM11) + (lM32 * rM21)) + (lM33 * rM31)) + (lM34 * rM41);
    result.row3.Y = (((lM31 * rM12) + (lM32 * rM22)) + (lM33 * rM32)) + (lM34 * rM42);
    result.row3.Z = (((lM31 * rM13) + (lM32 * rM23)) + (lM33 * rM33)) + (lM34 * rM43);
    result.row3.W = (((lM31 * rM14) + (lM32 * rM24)) + (lM33 * rM34)) + (lM34 * rM44);

    result.row4.X = (((lM41 * rM11) + (lM42 * rM21)) + (lM43 * rM31)) + (lM44 * rM41);
    result.row4.Y = (((lM41 * rM12) + (lM42 * rM22)) + (lM43 * rM32)) + (lM44 * rM42);
    result.row4.Z = (((lM41 * rM13) + (lM42 * rM23)) + (lM43 * rM33)) + (lM44 * rM43);
    result.row4.W = (((lM41 * rM14) + (lM42 * rM24)) + (lM43 * rM34)) + (lM44 * rM44);

    result
}
