use std::fs;

use rtstest::{
    render_queue_to_framebuffer, OpenGlRenderQueue, PlaceholderTexture, Tile, TileMap, TilePos,
};

fn main() {
    let mut map = TileMap::new(12, 12);
    map.set(TilePos { x: 3, y: 3 }, Tile::GoldMine);
    map.set(TilePos { x: 5, y: 4 }, Tile::Forest);

    let mut render_queue = OpenGlRenderQueue::default();
    render_queue.queue_placeholder_quad(PlaceholderTexture::GoldMine, 3.0, 3.0, 3.0);
    render_queue.queue_placeholder_quad(PlaceholderTexture::Forest, 7.0, 4.0, 2.0);
    render_queue.queue_placeholder_quad(PlaceholderTexture::Worker, 1.0, 1.0, 1.0);

    let frame = render_queue_to_framebuffer(&render_queue, 12, 12);
    let ppm = frame.to_ppm();
    let output_path = "target/render_preview.ppm";
    fs::create_dir_all("target").expect("failed to create target dir for render output");
    fs::write(output_path, ppm).expect("failed to write render preview ppm");

    println!(
        "MVP render: {} OpenGL placeholder квадрата(ов) отрисовано в {}.",
        render_queue.quads.len(),
        output_path
    );
}
