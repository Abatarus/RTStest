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
}
