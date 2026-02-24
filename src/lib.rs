pub const FIXED_TICK_MS: u64 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Gold,
    Wood,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourcePool {
    pub gold: u32,
    pub wood: u32,
}

impl ResourcePool {
    pub fn add(&mut self, kind: ResourceKind, amount: u32) {
        match kind {
            ResourceKind::Gold => self.gold += amount,
            ResourceKind::Wood => self.wood += amount,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilePos {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Ground,
    Blocked,
    GoldMine,
    Forest,
}

#[derive(Debug, Clone)]
pub struct TileMap {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl TileMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::Ground; width * height],
        }
    }

    pub fn set(&mut self, pos: TilePos, tile: Tile) {
        let idx = self.idx(pos);
        self.tiles[idx] = tile;
    }

    pub fn get(&self, pos: TilePos) -> Tile {
        self.tiles[self.idx(pos)]
    }

    pub fn is_buildable(&self, pos: TilePos) -> bool {
        matches!(self.get(pos), Tile::Ground)
    }

    fn idx(&self, pos: TilePos) -> usize {
        assert!(
            pos.x < self.width && pos.y < self.height,
            "tile out of map bounds"
        );
        pos.y * self.width + pos.x
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub f32, pub f32, pub f32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderTexture {
    Worker,
    Barracks,
    GoldMine,
    Forest,
}

impl PlaceholderTexture {
    pub fn color(self) -> Color {
        match self {
            PlaceholderTexture::Worker => Color(0.2, 0.4, 1.0),
            PlaceholderTexture::Barracks => Color(0.7, 0.2, 0.2),
            PlaceholderTexture::GoldMine => Color(1.0, 0.85, 0.1),
            PlaceholderTexture::Forest => Color(0.1, 0.7, 0.1),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct OpenGlRenderQueue {
    pub quads: Vec<Quad>,
}

impl OpenGlRenderQueue {
    pub fn queue_placeholder_quad(
        &mut self,
        texture: PlaceholderTexture,
        x: f32,
        y: f32,
        size: f32,
    ) {
        self.quads.push(Quad {
            x,
            y,
            size,
            color: texture.color(),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

#[derive(Debug, Clone)]
pub struct FrameBuffer {
    width: usize,
    height: usize,
    pixels: Vec<Rgb>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Rgb(0, 0, 0); width * height],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y * self.width + x;
        self.pixels[idx] = color;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Rgb {
        let idx = y * self.width + x;
        self.pixels[idx]
    }

    pub fn to_ppm(&self) -> String {
        let mut out = format!("P3\n{} {}\n255\n", self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let Rgb(r, g, b) = self.get_pixel(x, y);
                out.push_str(&format!("{} {} {} ", r, g, b));
            }
            out.push('\n');
        }
        out
    }
}

pub fn render_queue_to_framebuffer(
    queue: &OpenGlRenderQueue,
    width: usize,
    height: usize,
) -> FrameBuffer {
    let mut frame = FrameBuffer::new(width, height);

    for quad in &queue.quads {
        let rgb = color_to_rgb(quad.color);
        let min_x = quad.x.max(0.0) as usize;
        let min_y = quad.y.max(0.0) as usize;
        let max_x = (quad.x + quad.size).min(width as f32) as usize;
        let max_y = (quad.y + quad.size).min(height as f32) as usize;

        for y in min_y..max_y {
            for x in min_x..max_x {
                frame.set_pixel(x, y, rgb);
            }
        }
    }

    frame
}

pub fn build_demo_render_queue(time_s: f32) -> OpenGlRenderQueue {
    let mut render_queue = OpenGlRenderQueue::default();
    render_queue.queue_placeholder_quad(PlaceholderTexture::GoldMine, 3.0, 3.0, 3.0);
    render_queue.queue_placeholder_quad(PlaceholderTexture::Forest, 7.0, 4.0, 2.0);

    let worker_x = 1.0 + (time_s * 2.0).sin() * 0.75 + 1.0;
    let worker_y = 1.0 + (time_s * 1.5).cos() * 0.5 + 1.0;
    render_queue.queue_placeholder_quad(PlaceholderTexture::Worker, worker_x, worker_y, 1.0);

    render_queue
}

fn color_to_rgb(color: Color) -> Rgb {
    let Color(r, g, b) = color;
    Rgb(
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub resources: ResourcePool,
    pub map: TileMap,
    elapsed_ms: u64,
    pub ticks: u64,
}

impl GameState {
    pub fn new(map: TileMap) -> Self {
        Self {
            resources: ResourcePool::default(),
            map,
            elapsed_ms: 0,
            ticks: 0,
        }
    }

    pub fn harvest(&mut self, kind: ResourceKind, amount: u32) {
        self.resources.add(kind, amount);
    }

    pub fn advance_time(&mut self, delta_ms: u64) {
        self.elapsed_ms += delta_ms;
        while self.elapsed_ms >= FIXED_TICK_MS {
            self.elapsed_ms -= FIXED_TICK_MS;
            self.ticks += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harvest_increases_resource_pool() {
        let mut state = GameState::new(TileMap::new(8, 8));

        state.harvest(ResourceKind::Gold, 50);
        state.harvest(ResourceKind::Wood, 30);

        assert_eq!(state.resources.gold, 50);
        assert_eq!(state.resources.wood, 30);
    }

    #[test]
    fn building_allowed_only_on_ground_tiles() {
        let mut map = TileMap::new(4, 4);
        map.set(TilePos { x: 1, y: 1 }, Tile::Blocked);
        map.set(TilePos { x: 2, y: 2 }, Tile::Forest);

        assert!(map.is_buildable(TilePos { x: 0, y: 0 }));
        assert!(!map.is_buildable(TilePos { x: 1, y: 1 }));
        assert!(!map.is_buildable(TilePos { x: 2, y: 2 }));
    }

    #[test]
    fn fixed_tick_simulation_is_deterministic() {
        let mut state = GameState::new(TileMap::new(2, 2));
        state.advance_time(10);
        assert_eq!(state.ticks, 0);
        state.advance_time(40);
        assert_eq!(state.ticks, 1);
        state.advance_time(150);
        assert_eq!(state.ticks, 4);
    }

    #[test]
    fn opengl_placeholder_uses_colored_square_palette() {
        let mut queue = OpenGlRenderQueue::default();
        queue.queue_placeholder_quad(PlaceholderTexture::Worker, 1.0, 2.0, 16.0);

        assert_eq!(queue.quads.len(), 1);
        assert_eq!(queue.quads[0].color, Color(0.2, 0.4, 1.0));
        assert_eq!(queue.quads[0].size, 16.0);
    }

    #[test]
    fn render_queue_draws_colored_square_into_framebuffer() {
        let mut queue = OpenGlRenderQueue::default();
        queue.queue_placeholder_quad(PlaceholderTexture::GoldMine, 1.0, 1.0, 2.0);

        let frame = render_queue_to_framebuffer(&queue, 4, 4);

        assert_eq!(frame.get_pixel(0, 0), Rgb(0, 0, 0));
        assert_eq!(frame.get_pixel(1, 1), Rgb(255, 217, 26));
        assert_eq!(frame.get_pixel(2, 2), Rgb(255, 217, 26));
    }

    #[test]
    fn demo_render_queue_contains_expected_static_objects() {
        let queue = build_demo_render_queue(0.0);

        assert_eq!(queue.quads.len(), 3);
        assert_eq!(queue.quads[0].color, PlaceholderTexture::GoldMine.color());
        assert_eq!(queue.quads[1].color, PlaceholderTexture::Forest.color());
        assert_eq!(queue.quads[2].color, PlaceholderTexture::Worker.color());
    }

    #[test]
    fn demo_render_queue_animates_worker_position_over_time() {
        let first = build_demo_render_queue(0.0);
        let later = build_demo_render_queue(1.0);

        assert_ne!(first.quads[2].x, later.quads[2].x);
        assert_ne!(first.quads[2].y, later.quads[2].y);
    }

    #[test]
    fn ppm_export_starts_with_valid_header() {
        let mut frame = FrameBuffer::new(2, 1);
        frame.set_pixel(0, 0, Rgb(255, 0, 0));
        frame.set_pixel(1, 0, Rgb(0, 255, 0));

        let ppm = frame.to_ppm();
        assert!(ppm.starts_with("P3\n2 1\n255\n"));
        assert!(ppm.contains("255 0 0"));
        assert!(ppm.contains("0 255 0"));
    }
}
