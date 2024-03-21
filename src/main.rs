use bevy::{prelude::*, utils::HashSet, window::close_on_esc};
use bevy_pancam::{PanCam, PanCamPlugin};
use noise::{NoiseFn, Perlin};
use rand::Rng;

// Sprite
const SPRITE_SHEET_PATH: &str = "test.png";
const SPRITE_SCALE_FACTOR: usize = 5;
const TILE_W: usize = 6;
const TILE_H: usize = 8;
const SPRITE_SHEET_W: usize = 36 / TILE_W;
const SPRITE_SHEET_H: usize = 40 / TILE_H;

// Window
const GRID_COLS: usize = 200;
const GRID_ROWS: usize = 100;
const GEN_W: usize = GRID_COLS * TILE_W * SPRITE_SCALE_FACTOR;
const GEN_H: usize = GRID_ROWS * TILE_H * SPRITE_SCALE_FACTOR;

// Perlin
const NOISE_SCALE: f64 = 10.5;

// Colors
const BACKGROUND: Color = Color::rgb(0.5, 0.8, 0.8);
const GREEN: Color = Color::rgb(0.5, 0.8, 0.5);
const BROWN: Color = Color::rgb(0.4, 0.3, 0.25);

#[derive(Component)]
struct TileComponent;

struct Tile {
    pos: (i32, i32),
    sprite: usize,
    color: Color,
    z_index: i32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PanCamPlugin)
        .insert_resource(ClearColor(BACKGROUND))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .add_systems(Update, close_on_esc)
        .run();
}

fn handle_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    tiles_query: Query<Entity, With<TileComponent>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return
    }
    println!("TAB");

    for entity in tiles_query.iter() {
        commands.entity(entity).despawn();
    }
    gen_world(&mut commands, asset_server, &mut texture_atlas_layouts);
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_xyz(GEN_W as f32 / 2.0, GEN_H as f32 / 2.0, 0.0),
            ..Default::default()
        })
        .insert(PanCam::default());

    gen_world(&mut commands, asset_server, &mut texture_atlas_layouts);
}

fn gen_world(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut rng = rand::thread_rng();
    let perlin = Perlin::new(rng.gen());

    let texture = asset_server.load(SPRITE_SHEET_PATH);
    let layout = TextureAtlasLayout::from_grid(
        Vec2::new( TILE_W as f32, TILE_H as f32), 
        SPRITE_SHEET_W, 
        SPRITE_SHEET_H, 
        None, 
        None
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let mut tiles = Vec::new();
    let mut occupied = HashSet::new();
    for x in 0..GRID_COLS {
        for y in 0..GRID_ROWS {
            let noise_val = perlin.get([x as f64 / NOISE_SCALE, y as f64 / NOISE_SCALE]);
            let (x, y) = (x as i32, y as i32);
            let choice = rng.gen_range(0.0..1.0);

            // Ground
            if noise_val > 0.2 {
                occupied.insert((x, y));
            }

            // Mountains
            if noise_val > 0.3 && noise_val < 0.31 {
                tiles.push(Tile::new((x, y), 1, 1, Color::BEIGE));
            }

            // Trees
            if noise_val > 0.35 && noise_val < 0.6 {

                if choice > 0.9 {
                    tiles.push(Tile::new((x, y), rng.gen_range(7..=9), 1, GREEN));
                } else if choice > 0.8 {
                    tiles.push(Tile::new((x, y), 6, 1, GREEN));
                }
            }

            // Bones
            if noise_val > 0.6 && noise_val < 0.7 && choice > 0.98 {
                tiles.push(Tile::new((x, y), rng.gen_range(18..=19), 1, Color::GRAY));
            }

            // House
            if noise_val > 0.7 && choice > 0.98 {
                let house_tile = if rng.gen_range(0.0..1.0) > 0.85 { 12 } else { 13 };
                tiles.push(Tile::new((x, y), house_tile, 1, BROWN));
            }
        }
    }

    for (x, y) in occupied.iter() {
        let (tile, nei_count) = get_tile((*x, *y), &occupied);

        if nei_count <= 1 {
            continue;
        }
        tiles.push(Tile::new((*x, *y), tile, 0, Color::BEIGE));
    }

    for tile in tiles.iter() {
        let (x, y) = tile.pos;
        let (x, y) = grid_to_world(x, y);
        //let (x, y) = center_to_top_left(x, y);

        commands.spawn((
            SpriteSheetBundle {
                sprite: Sprite {
                    color: tile.color,
                    ..default()
                },
                texture: texture.clone(),
                atlas: TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: tile.sprite,
                },
                transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR as f32))
                    .with_translation(Vec3::new(x, y, tile.z_index as f32)),
                ..default()
            },
            TileComponent
        ));
    }
}

fn get_tile((x, y): (i32, i32), occupied: &HashSet<(i32, i32)>) -> (usize, i32) {
    // [TOP, RIGHT, BOTTOM, LEFT]
    let nei_options = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    let mut nei = [1, 1, 1, 1];
    let mut nei_count = 0;

    for (idx, (i, j)) in nei_options.into_iter().enumerate() {
        if occupied.contains(&(x + i, y + j)) {
            nei_count += 1;
            continue;
        }

        nei[idx] = 0;

    }

    let tile = match nei {
        [0, 1, 1, 0] => 1,
        [0, 0, 1, 1] => 2,
        [1, 1, 0, 0] => 3,
        [1, 0, 0, 1] => 4,
        _ => 0
    };
    (tile, nei_count)
}

fn grid_to_world(x: i32, y: i32) -> (f32, f32) {
    (
        x as f32 * TILE_W as f32 * SPRITE_SCALE_FACTOR as f32,
        y as f32 * TILE_H as f32 * SPRITE_SCALE_FACTOR as f32,
    )
}

impl Tile {
    fn new(pos: (i32, i32), sprite: usize, z_index: i32, color: Color) -> Self {
        Self { pos, sprite, z_index, color }
    }
}
