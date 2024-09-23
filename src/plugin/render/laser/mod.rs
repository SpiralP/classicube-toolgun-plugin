pub mod texture;

use std::os::raw::c_float;

use classicube_sys::{
    Camera, Gfx, Gfx_LoadMatrix, Gfx_SetAlphaTest, Gfx_SetFaceCulling, Gfx_SetTexturing, Matrix,
    MatrixType__MATRIX_VIEW, OwnedTexture, Vec3, MATH_DEG2RAD,
};
use texture::create_texture;
use tracing::debug;

use super::{context::vertex_buffer::Texture_Render, render_hook::renderable::Renderable};

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
        let block_width = (self.end_pos - self.start_pos).length_squared().sqrt();

        let dir = (self.end_pos - self.start_pos).normalize();
        let pitch = dir.Y.asin();
        let yaw = dir.X.atan2(-dir.Z);

        let height = self.texture.as_texture().Height as f32;
        let width = self.texture.as_texture().Width as f32;

        let scale = Matrix::scale((0.5 * block_width * 2.0) / width, 0.5 / height, 1.0);
        let translation = Matrix::translate(self.start_pos.X, self.start_pos.Y, self.start_pos.Z);

        let orientation = unsafe {
            let camera = &*Camera.Active;
            let get_orientation = camera.GetOrientation.unwrap();
            get_orientation()
        };
        let transform = scale
            * Matrix::rotate_z(pitch)
            * Matrix::rotate_y(-yaw + 90.0 * MATH_DEG2RAD as f32)
            * Matrix::rotate_x(0.0)
            * translation;

        for front in [true, false] {
            unsafe {
                let m = transform * Gfx.View;
                Gfx_LoadMatrix(MatrixType__MATRIX_VIEW, &m);

                Gfx_SetAlphaTest(1);
                Gfx_SetTexturing(1);
                Gfx_SetFaceCulling(1);

                Texture_Render(self.texture.as_texture_mut(), front);

                Gfx_SetFaceCulling(0);

                Gfx_LoadMatrix(MatrixType__MATRIX_VIEW, &Gfx.View);
            }
        }
    }
}

impl Renderable for Laser {
    fn render(&mut self) {
        self.render_inner();
    }
}
