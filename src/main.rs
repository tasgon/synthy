extern crate ggez;

mod imgui_wrapper;

use crate::imgui_wrapper::ImGuiWrapper;
use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics;
use ggez::graphics::DrawParam;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
use imgui::im_str;
use std::convert::TryInto;
use std::sync::Arc;
use std::time::Instant;

mod assets;
//mod event_mgr;
mod keyboard;
mod midi_interpreter;
mod song;

use keyboard::{Key, KeyType};

struct MainState {
    imgui_wrapper: ImGuiWrapper,
    hidpi_factor: f32,
    main_assets: Arc<assets::Assets>,
    board: keyboard::Keyboard,
    current_song: song::Song,
    reference: Instant,
    show_ui: bool,
}

impl MainState {
    fn new(mut ctx: &mut Context, hidpi_factor: f32) -> GameResult<MainState> {
        let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
        let main_assets = Arc::new(assets::Assets::new(ctx, &std::path::Path::new("assets")));
        let board = keyboard::Keyboard::new(main_assets.clone());
        let current_song = song::Song::new("/home/me/Downloads/Again.mid");
        let reference = Instant::now();
        let s = MainState {
            imgui_wrapper,
            hidpi_factor,
            main_assets,
            board,
            current_song,
            reference,
            show_ui: true,
        };
        Ok(s)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.current_song.update(&self.reference);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(
            ctx,
            graphics::Color {
                r: 0.3,
                g: 1.0,
                b: 0.3,
                a: 1.0,
            },
        );

        // Render game stuff
        {
            let rect = ggez::graphics::screen_coordinates(ctx);
            let hfac = rect.h * 0.85;

            let width_scale = rect.w / (self.main_assets.white_key.width() as f32 * 52f32);
            let keymap = &self.main_assets.keymap;
            for tile in self.current_song.active_tiles.iter() {
                let key: Key = keymap[tile.note as usize];
                let fac = tile.vertical_height(hfac) / self.main_assets.white_key.height() as f32;
                let dest = na::Point2::new(
                    key.offset.x * width_scale, //TODO: clean this up
                    tile.vertical_position(&self.reference, rect.h),
                );
                graphics::draw(
                    ctx,
                    match key.key_type {
                        KeyType::WHITE => &self.main_assets.white_key,
                        KeyType::BLACK => &self.main_assets.black_key,
                    },
                    DrawParam::new()
                        .dest(dest)
                        .scale(na::Vector2::new(width_scale, fac)),
                )
                .unwrap();
            }

            self.board.draw_piano(
                ctx,
                (
                    na::Point2::new(0.0, hfac),
                    graphics::Color::new(1.0, 1.0, 1.0, 0.0),
                ),
            );
        }

        // Render game ui
        if self.show_ui {
            let fps = ggez::timer::fps(ctx);
            let i: i32 = song::deltat().as_millis().try_into().unwrap();
            let rt = &mut self.reference;
            self.imgui_wrapper
                .render(ctx, self.hidpi_factor, move |ui| {
                    imgui::Window::new(im_str!("Hello world"))
                        .size([300.0, 600.0], imgui::Condition::FirstUseEver)
                        .position([50.0, 50.0], imgui::Condition::FirstUseEver)
                        .build(ui, || {
                            let mut j: i32 = i;
                            ui.text(im_str!("Hi from this label!"));
                            ui.text(im_str!("FPS: {:.2}", fps));
                            ui.input_int(im_str!("dt"), &mut j).build();
                            if i != j {
                                song::set_deltat(j as u64);
                            }
                            if ui.small_button(im_str!("Reset timer")) {
                                *rt = Instant::now();
                            }
                        });
                });
        }

        graphics::draw(
            ctx,
            &graphics::Text::new("fragment: F"),
            (na::Point2::new(10.0, 10.0),),
        )
        .unwrap();

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((
            button == MouseButton::Left,
            button == MouseButton::Right,
            button == MouseButton::Middle,
        ));
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((false, false, false));
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::Escape {
            self.show_ui = !self.show_ui;
        }
        self.imgui_wrapper.update_key_down(keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        self.imgui_wrapper.update_key_up(keycode, keymods);
    }

    fn text_input_event(&mut self, _ctx: &mut Context, val: char) {
        self.imgui_wrapper.update_text(val);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height))
            .unwrap();
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.imgui_wrapper.update_scroll(x, y);
    }
}

pub fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("synthy", "telastrus")
        .window_setup(conf::WindowSetup::default().title("synthy."))
        .window_mode(
            conf::WindowMode::default().resizable(true), /*.dimensions(750.0, 500.0)*/
        );
    let (ref mut ctx, event_loop) = &mut cb.build()?;

    let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor() as f32;
    println!("main hidpi_factor = {}", hidpi_factor);

    let state = &mut MainState::new(ctx, hidpi_factor)?;

    event::run(ctx, event_loop, state)
}
