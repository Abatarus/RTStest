use rtstest::{OpenGlRenderQueue, PlaceholderTexture, Tile, TileMap, TilePos};

fn main() {
    let mut map = TileMap::new(12, 12);
    map.set(TilePos { x: 3, y: 3 }, Tile::GoldMine);
    map.set(TilePos { x: 5, y: 4 }, Tile::Forest);

    let mut render_queue = OpenGlRenderQueue::default();
    render_queue.queue_placeholder_quad(PlaceholderTexture::GoldMine, 3.0, 3.0, 1.0);
    render_queue.queue_placeholder_quad(PlaceholderTexture::Forest, 5.0, 4.0, 1.0);
    render_queue.queue_placeholder_quad(PlaceholderTexture::Worker, 1.0, 1.0, 1.0);

    println!(
        "MVP bootstrap: {} OpenGL placeholder квадрата(ов) подготовлено к рендеру.",
        render_queue.quads.len()
    );
    println!("Следующий шаг: подключить реальный OpenGL backend и отрисовать очередь квадов.");
}
