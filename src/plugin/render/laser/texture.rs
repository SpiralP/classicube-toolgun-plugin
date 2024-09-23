use std::os::raw::c_int;

use classicube_sys::{
    cc_int16, Bitmap, Context2D, Context2D_DrawPixels, OwnedContext2D, OwnedTexture,
    PackedCol_Make, TextureRec,
};
use tracing::{debug, warn};

use crate::textures::{LIGHTNING_FRAME_HEIGHT, LIGHTNING_FRAME_PIXELS, LIGHTNING_FRAME_WIDTH};

const BLOCK_WIDTH: f32 = 16.0;

/// returns (front, back)
#[tracing::instrument]
pub fn create_texture(block_width: f32) -> OwnedTexture {
    debug!("");

    let (mut context_2d, width, height) = unsafe {
        let width = LIGHTNING_FRAME_WIDTH as c_int;
        let height = LIGHTNING_FRAME_HEIGHT as c_int;
        debug!(?width, ?height);

        let mut context_2d = OwnedContext2D::new_pow_of_2(width, height, 0x0000_0000);

        draw_parts(context_2d.as_context_2d_mut(), width, height);

        (context_2d, width, height)
    };

    let u2 =
        (block_width * 2.0 * (width as f32 / BLOCK_WIDTH)) / context_2d.as_bitmap().width as f32;
    let v2 = height as f32 / context_2d.as_bitmap().height as f32;

    let front_texture = OwnedTexture::new(
        context_2d.as_bitmap_mut(),
        (0, -(height as cc_int16)),
        (width as _, height as _),
        TextureRec {
            U1: 0.0,
            V1: 0.0,
            U2: u2,
            V2: v2,
        },
    );

    front_texture
}

unsafe fn draw_parts(context: &mut Context2D, width: c_int, height: c_int) {
    let mut lightning_frame_width = LIGHTNING_FRAME_PIXELS.to_vec();
    // lightning_frame_width[0] = PackedCol_Make(255, 0, 0, 255);
    // lightning_frame_width[1] = PackedCol_Make(0, 255, 0, 255);
    // lightning_frame_width[2] = PackedCol_Make(0, 0, 255, 255);
    // lightning_frame_width[3] = 0xFFFF_0000;
    // lightning_frame_width[4] = 0xFF00_FF00;
    // lightning_frame_width[5] = 0xFF00_00FF;
    Context2D_DrawPixels(
        context,
        0,
        0,
        &mut Bitmap {
            scan0: lightning_frame_width.as_mut_ptr(),
            width,
            height,
        },
    );
}
