pub mod texture;

use std::os::raw::c_float;

use classicube_sys::{
    Gfx, Gfx_LoadMatrix, Gfx_SetAlphaTest, Gfx_SetFaceCulling, Gfx_SetTexturing, Matrix,
    MatrixType__MATRIX_VIEW, OwnedTexture, Vec3, MATH_DEG2RAD,
};
use texture::create_texture;
use tracing::debug;

use super::{context::vertex_buffer::Texture_Render, render_hook::renderable::Renderable};

pub struct Laser {
    texture: OwnedTexture,
    transform: Matrix,
}

impl Laser {
    pub fn new(start_pos: Vec3, end_pos: Vec3) -> Self {
        let block_width = 8.0;

        let texture = create_texture(block_width);
        let height = texture.as_texture().Height as f32;
        let width = texture.as_texture().Width as f32;

        let scale = Vec3::create((0.5 * block_width * 2.0) / width, 0.5 / height, 1.0);

        let translation = Matrix::translate(start_pos.X, start_pos.Y, start_pos.Z);
        let scale = Matrix::scale(scale.X, scale.Y, scale.Z);

        let rotation_X = 0.0;
        let rotation_Y = 0.0;
        let transform = scale
            * Matrix::rotate_z(180.0 * MATH_DEG2RAD as c_float)
            * Matrix::rotate_x(-rotation_X * MATH_DEG2RAD as c_float)
            * Matrix::rotate_y(-rotation_Y * MATH_DEG2RAD as c_float)
            * translation;

        Self { texture, transform }
    }

    fn render_inner(&mut self) {
        for front in [true, false] {
            unsafe {
                let m = self.transform * Gfx.View;
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
