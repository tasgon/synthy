use resvg::backend_raqote;
use resvg::prelude::*;
use resvg::raqote::DrawTarget;
use std::convert::TryInto;
use std::path::{Path, PathBuf};

use arrayvec::ArrayVec;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::Image;
use ggez::nalgebra as na;

use crate::keyboard::{BaseKeyboard, Key, KeyType, LAYOUT};

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
    pub white_key_active: Image,
    pub black_key_active: Image,
    pub keyboard: BaseKeyboard,
    pub keymap: ArrayVec<[Key; 100]>,
}

impl Assets {
    pub fn new(ctx: &mut ggez::Context, p: &Path) -> Self {
        let white_key = render_img(ctx, p.join("white_key.svg"));
        let black_key = render_img(ctx, p.join("black_key.svg"));
        let white_key_active = render_img(ctx, p.join("white_key_active.svg"));
        let black_key_active = render_img(ctx, p.join("black_key_active.svg"));
        let (keyboard, keymap) = Self::gen_piano(&white_key, &black_key);
        Self {
            white_key,
            black_key,
            white_key_active,
            black_key_active,
            keyboard,
            keymap,
        }
    }

    pub fn gen_piano(
        white_key: &Image,
        black_key: &Image,
    ) -> ((SpriteBatch, SpriteBatch), ArrayVec<[Key; 100]>) {
        let mut pos = 0;
        let mut white_batch = SpriteBatch::new(white_key.clone());
        let mut black_batch = SpriteBatch::new(black_key.clone());
        let white_dist = white_key.width();
        let black_dist = black_key.width() / 2;
        let mut keymap: ArrayVec<[Key; 100]> = ArrayVec::new();
        for i in LAYOUT.chars() {
            if i == 'W' {
                //println!("{}", pos);
                let pt = na::Point2::new(pos as f32, 0.0);
                white_batch.add((pt,));
                pos += white_dist;
                keymap.push(Key {
                    key_type: KeyType::WHITE,
                    offset: pt,
                });
            } else {
                let pt = na::Point2::new((pos - black_dist) as f32, 0.0);
                black_batch.add((pt,));
                keymap.push(Key {
                    key_type: KeyType::BLACK,
                    offset: pt,
                });
            }
        }
        ((white_batch, black_batch), keymap)
    }
}
