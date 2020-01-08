use resvg::backend_raqote;
use resvg::prelude::*;
use resvg::raqote::DrawTarget;
use std::convert::TryInto;
use std::path::{Path, PathBuf};

use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::Canvas;
use ggez::graphics::Image;
use ggez::nalgebra as na;

// Hard-coding the layout for now.
// TODO: be smart about this.
static LAYOUT: &str =
    "WBWWBWBWWBWBWBWWBWBWWBWBWBWWBWBWWBWBWBWWBWBWWBWBWBWWBWBWWBWBWBWWBWBWWBWBWBWWBWBWWBWBWBWW";

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
    keyboard: (SpriteBatch, SpriteBatch),
}

impl Assets {
    pub fn new(ctx: &mut ggez::Context, p: &Path) -> Self {
        let white_key = render_img(ctx, p.join("white_key.svg"));
        let black_key = render_img(ctx, p.join("black_key.svg"));
        let keyboard = Self::gen_piano(ctx, &white_key, &black_key);
        Self {
            white_key,
            black_key,
            keyboard,
        }
    }

    fn gen_piano(
        ctx: &mut ggez::Context,
        white_key: &Image,
        black_key: &Image,
    ) -> (SpriteBatch, SpriteBatch) {
        let mut pos = 0;
        let mut white_batch = SpriteBatch::new(white_key.clone());
        let mut black_batch = SpriteBatch::new(black_key.clone());
        let white_dist = white_key.width();
        let black_dist = black_key.width() / 2;
        println!("{}, {}", white_dist, black_dist * 2);
        for i in LAYOUT.chars() {
            if i == 'W' {
                //println!("{}", pos);
                white_batch.add((na::Point2::new(pos as f32, 0.0),));
                pos += white_dist;
            } else {
                black_batch.add((na::Point2::new((pos - black_dist) as f32, 0.0),));
            }
        }
        (white_batch, black_batch)
    }

    pub fn draw_piano<T: Into<ggez::graphics::DrawParam>>(
        &self,
        ctx: &mut ggez::Context,
        params: T,
    ) {
        let rect = ggez::graphics::screen_coordinates(ctx);
        let width = self.white_key.width() * 52;
        let height = self.white_key.height() as f32;
        let (white, black) = &self.keyboard;
        let mut p: ggez::graphics::DrawParam = params.into();
        p.scale = na::Vector2::new(rect.w / (width as f32), rect.w * 0.1 / height).into();
        ggez::graphics::draw(ctx, white, p.clone()).unwrap();
        ggez::graphics::draw(ctx, black, p).unwrap();
    }
}
