use std::time::{Duration, Instant};

use rtstest::{build_demo_render_queue, PlaceholderTexture};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 720;
const WORLD_WIDTH: f32 = 12.0;
const WORLD_HEIGHT: f32 = 12.0;

fn main() {
    let sdl_context = sdl2::init().expect("failed to init SDL2");
    let video = sdl_context
        .video()
        .expect("failed to init SDL2 video subsystem");

    let window = video
        .window("RTStest realtime SDL render", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .expect("failed to create SDL window");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("failed to create SDL canvas");

    let mut event_pump = sdl_context
        .event_pump()
        .expect("failed to create SDL event pump");

    let start = Instant::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let time_s = start.elapsed().as_secs_f32();
        let queue = build_demo_render_queue(time_s);

        canvas.set_draw_color(Color::RGB(16, 18, 24));
        canvas.clear();

        for quad in &queue.quads {
            let tex = if quad.color == PlaceholderTexture::Worker.color() {
                PlaceholderTexture::Worker
            } else if quad.color == PlaceholderTexture::Barracks.color() {
                PlaceholderTexture::Barracks
            } else if quad.color == PlaceholderTexture::GoldMine.color() {
                PlaceholderTexture::GoldMine
            } else {
                PlaceholderTexture::Forest
            };

            let color = to_sdl_color(tex);
            canvas.set_draw_color(color);

            let (x, y, w, h) = world_quad_to_screen(quad.x, quad.y, quad.size, quad.size);
            let rect = Rect::new(x, y, w, h);
            canvas
                .fill_rect(rect)
                .expect("failed to draw placeholder quad");
        }

        canvas.present();
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn to_sdl_color(texture: PlaceholderTexture) -> Color {
    let c = texture.color();
    Color::RGB(
        (c.0.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.1.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.2.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn world_quad_to_screen(x: f32, y: f32, w: f32, h: f32) -> (i32, i32, u32, u32) {
    let sx = (x / WORLD_WIDTH * WINDOW_WIDTH as f32).round() as i32;
    let sy = (y / WORLD_HEIGHT * WINDOW_HEIGHT as f32).round() as i32;
    let sw = (w / WORLD_WIDTH * WINDOW_WIDTH as f32).max(1.0).round() as u32;
    let sh = (h / WORLD_HEIGHT * WINDOW_HEIGHT as f32).max(1.0).round() as u32;
    (sx, sy, sw, sh)
}
