pub mod texture;

use std::{f32::consts::PI, os::raw::c_float};

use classicube_sys::{
    Camera, Gfx, Gfx_LoadMatrix, Gfx_SetAlphaBlending, Gfx_SetAlphaTest, Gfx_SetFaceCulling,
    Gfx_SetFog, Gfx_SetTexturing, Matrix, MatrixType__MATRIX_VIEW, OwnedTexture, Vec3,
    MATH_DEG2RAD, MATH_RAD2DEG,
};
use nalgebra::{center, distance, AbstractRotation, Point3, Rotation3, Vector3};
use texture::create_texture;
use tracing::debug;

use super::{context::vertex_buffer::Texture_Render, render_hook::renderable::Renderable};

pub fn vec3_to_point3(v: &Vec3) -> Point3<f32> {
    Point3::new(v.X, v.Y, v.Z)
}

pub struct Laser {
    start_pos: Vec3,
    end_pos: Vec3,
    texture: OwnedTexture,
}

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

        let scale = Matrix::scale((0.5 * block_width * 2.0) / width, 0.5 / height, 1.0);
        let translation = Matrix::translate(self.start_pos.X, self.start_pos.Y, self.start_pos.Z);

        let dir = (end_pos - start_pos).normalize();
        let pitch = dir.y.asin();
        let yaw = -dir.x.atan2(-dir.z);

        let eye_dir = (center(&start_pos, &end_pos) - eye_pos).normalize();
        let eye_dir = dir.cross(&eye_dir).normalize();
        let x = eye_dir.dot(&-Vector3::z());
        let z = eye_dir.dot(&Vector3::x());
        let mut eye_yaw = yaw + x.atan2(-z);
        if pitch > 0.0 {
            eye_yaw *= -1.0;
        }
        let transform = scale
            // angle 0,0 means the plane is facing that direction
            * Matrix::rotate_x(eye_yaw + 90.0f32.to_radians())
            * Matrix::rotate_z(pitch)
            * Matrix::rotate_y(yaw + 90.0f32.to_radians())
            * translation;

        unsafe {
            let m = transform * Gfx.View;
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

impl Renderable for Laser {
    fn render(&mut self) {
        self.render_inner();
    }
}

#[test]
fn test_math() {
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
            (
                (64.0, 40.0, 64.0),
                (70.0, 40.0, 64.0),
                6.0,
                (1.0, 0.0, 0.0),
                0.0,
                -90.0,
            ),
            (
                (64.0, 40.0, 64.0),
                (64.0, 50.0, 64.0),
                10.0,
                (0.0, 1.0, 0.0),
                90.0,
                -180.0,
            ),
            (
                (64.0, 40.0, 64.0),
                (74.0, 40.0, 54.0),
                14.142136,
                (0.70710677, 0.0, -0.70710677),
                0.0,
                -45.0,
            ),
            (
                (64.0, 40.0, 64.0),
                (64.0, 40.0, 70.0),
                6.0,
                (0.0, 0.0, 1.0),
                0.0,
                -180.0,
            ),
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
    }
}
