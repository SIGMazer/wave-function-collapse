use raylib::prelude::*;
use raylib::core::texture::Image;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

#[derive(Debug,Clone)]
struct Tile {
    img: Image,
    up:[usize; 3],
    down:[usize; 3],
    left:[usize; 3],
    right:[usize; 3],
}

#[derive(Debug, Default, Clone)]
struct Cell {
    tile: Option<Tile>,
    collapsed: bool,
    available: Vec<i32>,
    current: i32,
}

trait Sum {
    fn sum(&self) -> usize;
}
impl Sum for Color {
    fn sum(&self) -> usize {
        self.r as usize + self.g as usize + self.b as usize
    }
}

impl Tile {
    fn new(img: Image) -> Self {
        Self {
            img,
            up: [0, 0, 0],
            down: [0, 0, 0],
            left: [0, 0, 0],
            right: [0, 0, 0],
        }
    }
    fn set_rules(&mut self) {
        let image_data = self.img.get_image_data();
        let image_width = self.img.width;
        let image_height = self.img.height;

        // top and bottom pixels of image 
        for x in 0..image_width/ 3 {
            let pt1 = image_width / 3 * 0 + x;
            let pt2 = image_width / 3 * 1 + x;
            let pt3 = image_width / 3 * 2 + x;
            let pb1 = image_width * (image_height - 1) + pt1;
            let pb2 = image_width * (image_height - 1) + pt2;
            let pb3 = image_width * (image_height - 1) + pt3;

            self.up[0] += image_data[pt1 as usize].sum();
            self.up[1] += image_data[pt2 as usize].sum();
            self.up[2] += image_data[pt3 as usize].sum();
            self.down[0] += image_data[pb1 as usize].sum();
            self.down[1] += image_data[pb2 as usize].sum();
            self.down[2] += image_data[pb3 as usize].sum();

        }
        // left and right pixels of image 
        for y in 0..image_height/ 3 {
            let pl1 = image_width * y + 0;
            let pl2 = image_width * y + image_width / 3;
            let pl3 = image_width * y + image_width / 3 * 2;
            let pr1 = image_width * y + image_width - 1;
            let pr2 = image_width * y + image_width - 1 - image_width / 3;
            let pr3 = image_width * y + image_width - 1 - image_width / 3 * 2;

            self.left[0] += image_data[pl1 as usize].sum();
            self.left[1] += image_data[pl2 as usize].sum();
            self.left[2] += image_data[pl3 as usize].sum();
            self.right[0] += image_data[pr1 as usize].sum();
            self.right[1] += image_data[pr2 as usize].sum();
            self.right[2] += image_data[pr3 as usize].sum();
        }
    }
    fn rotate(&mut self) -> Self {
        let mut new_img = self.img.clone();
        new_img.rotate_ccw();
        let mut new_tile = Tile::new(new_img);
        new_tile.set_rules();
        new_tile
    }
}

fn init_grid(grid: &mut Vec<Vec<Cell>>, tiles: &Vec<Tile>) {
    for y in 0..grid.len() {
        for x in 0..grid[y].len() {
            grid[y][x].available = tiles.iter().enumerate().map(|(i, _)| i as i32).collect();
        }
    }
}

fn collapse(grid: &mut Vec<Vec<Cell>>, tiles: &Vec<Tile>) {
    let mut rng = ChaCha20Rng::from_entropy();

    let y = rng.gen_range(0..grid.len());
    let x = rng.gen_range(0..grid[y].len());


    if !grid[y][x].collapsed && grid[y][x].available.len() > 1 {
        let random_index = rng.gen_range(0..grid[y][x].available.len());
        let selected_tile_index = grid[y][x].available[random_index] as usize;
        grid[y][x].current = selected_tile_index as i32;

        grid[y][x].tile = Some(tiles[selected_tile_index].clone());
        grid[y][x].collapsed = true;

        // Update neighbors
        if y > 0 && !grid[y - 1][x].collapsed {
            grid[y - 1][x].available.retain(|&i| i != selected_tile_index as i32);
        }
        if y < grid.len() - 1 && !grid[y + 1][x].collapsed {
            grid[y + 1][x].available.retain(|&i| i != selected_tile_index as i32);
        }
        if x > 0 && !grid[y][x - 1].collapsed {
            grid[y][x - 1].available.retain(|&i| i != selected_tile_index as i32);
        }
        if x < grid[y].len() - 1 && !grid[y][x + 1].collapsed {
            grid[y][x + 1].available.retain(|&i| i != selected_tile_index as i32);
        }
    }

}


fn draw_grid(grid: &Vec<Vec<Cell>>, d: &mut RaylibDrawHandle, textures: &Vec<Texture2D>) {
    for y in 0..grid.len() {
        for x in 0..grid[y].len() {
            if let Some(tile) = &grid[y][x].tile {
                let available = grid[y][x].current as usize;
                d.draw_texture_ex(
                    &textures[available],
                    Vector2::new(x as f32 * tile.img.width as f32, y as f32 * tile.img.height as f32),
                    0.0,
                    1.0,
                    Color::WHITE,
                );
            }
        }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Hello, World")
        .build();
    

    let mut img = Image::load_image("tiles/cave.png").unwrap();
    let mut tiles = Vec::new();

    let grid_width = rl.get_screen_width() / img.width;
    let grid_height = rl.get_screen_height() / img.height;
    let mut grid = vec![vec![Cell::default(); grid_width as usize]; grid_height as usize];



    for _ in 0..4 {
        let mut tile = Tile::new(img.clone());
        tile.set_rules();
        tiles.push(tile);
        img.rotate_ccw();
    }

    init_grid(&mut grid, &tiles);

    let mut textures = Vec::new();
    for tile in tiles.iter() {
        textures.push(rl.load_texture_from_image(&thread, &tile.img).unwrap());
    }

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        collapse(&mut grid, &tiles);
        draw_grid(&grid, &mut d, &textures);
    }
}

