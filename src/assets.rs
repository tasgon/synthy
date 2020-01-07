use resvg::backend_raqote;
use resvg::prelude::*;
use resvg::raqote::DrawTarget;
use std::convert::TryInto;
use std::path::{Path, PathBuf};

use ggez::graphics::Image;

pub fn render_svg(path: PathBuf) -> DrawTarget {
    let mut opt = resvg::Options::default();
    opt.usvg.path = Some(path.clone());
    let rtree = usvg::Tree::from_file(&path, &opt.usvg).unwrap();
    let img: DrawTarget = backend_raqote::render_to_image(&rtree, &opt).unwrap();
    img
}

pub fn render_img(ctx: &mut ggez::Context, path: PathBuf) -> Image {
    let mut img = render_svg(path);
    Image::from_rgba8(
        ctx,
        img.width().try_into().unwrap(),
        img.height().try_into().unwrap(),
        img.get_data_u8_mut(),
    )
    .unwrap()
}

pub struct Assets {
    pub white_key: Image,
    pub black_key: Image,
}

impl Assets {
    pub fn new(ctx: &mut ggez::Context, p: &Path) -> Self {
        Self {
            white_key: render_img(ctx, p.join("white_key.svg")),
            black_key: render_img(ctx, p.join("black_key.svg")),
        }
    }
}
