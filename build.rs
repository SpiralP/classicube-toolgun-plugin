use std::{
    env,
    fs::{read_dir, File},
    io::Write,
    path::{Path, PathBuf},
};

use classicube_sys::{PackedCol, PackedCol_Make};

const IMAGE_DIR: &str = "textures";

fn main() {
    let mut code_parts = Vec::new();

    for dir in read_dir(IMAGE_DIR).unwrap() {
        let dir = dir.unwrap();
        let metadata = dir.metadata().unwrap();
        if metadata.is_file() && dir.path().extension().unwrap() == "png" {
            let (width, height, pixels) = get_pixels(dir.path());
            let name = dir.path().file_stem().unwrap().to_ascii_uppercase();
            let name = name.to_string_lossy();

            code_parts.push(format!("pub const {}_WIDTH: u32 = {};", name, width));
            code_parts.push(format!("pub const {}_HEIGHT: u32 = {};", name, height));
            code_parts.push(format!(
                "pub const {}_PIXELS: [::classicube_sys::PackedCol; {}] = {:?};",
                name,
                pixels.len(),
                pixels
            ));
        }
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join(format!("{}.rs", IMAGE_DIR));
    let mut rust_code_file = File::create(path).unwrap();
    writeln!(rust_code_file, "{}", code_parts.join("\n")).unwrap();
}

fn get_pixels(path: PathBuf) -> (u32, u32, Vec<PackedCol>) {
    println!("{:?}", path);

    // The decoder is a build for reader and can be used to set various decoding options
    // via `Transformations`. The default output transformation is `Transformations::EXPAND
    // | Transformations::STRIP_ALPHA`.
    let decoder = png::Decoder::new(File::open(path).unwrap());
    let mut reader = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buf = vec![0; reader.output_buffer_size()];
    // Read the next frame. An APNG might contain multiple frames.
    let info = reader.next_frame(&mut buf).unwrap();
    // Grab the bytes of the image.
    let bytes = &buf[..info.buffer_size()];

    assert_eq!(info.bit_depth, png::BitDepth::Eight);

    // TODO fix PackedCol_Make on linux
    fn color(r: u8, g: u8, b: u8, a: u8) -> PackedCol {
        #[cfg(not(target_os = "linux"))]
        {
            PackedCol_Make(r, g, b, a)
        }

        #[cfg(target_os = "linux")]
        {
            PackedCol_Make(b, g, r, a)
        }
    }

    (
        info.width,
        info.height,
        match info.color_type {
            png::ColorType::Rgb => bytes
                .chunks(3)
                .map(|c| color(c[0], c[1], c[2], 255))
                .collect(),
            png::ColorType::Rgba => bytes
                .chunks(4)
                .map(|c| color(c[0], c[1], c[2], c[3]))
                .collect(),

            other => {
                panic!("unsupported ColorType {:?}", other)
            }
        },
    )
}
