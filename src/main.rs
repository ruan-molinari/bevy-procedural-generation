use std::{borrow::Borrow, isize};

use bevy::{prelude::*, reflect::Array, utils::HashSet, window::close_on_esc};
use bevy_pancam::{PanCam, PanCamPlugin};
use noise::{NoiseFn, Perlin};
use rand::Rng;

// Sprite
const SPRITE_SHEET_PATH: &str = "test.png";
const SPRITE_SCALE_FACTOR: f32 = 5.0;
const TILE_W: f32 = 6.0;
const TILE_H: f32 = 8.0;

// window
const GRID_COLS: usize = 200;
const GRID_ROWS: usize = 100;

// Perlin
const NOISE_SCALE: f64 = 10.5;

// Colors
const BACKGROUND: Color = Color::rgb(0.7, 0.7, 0.7);
const SAND: Color = Color::rgb(1.0, 1.0, 0.9);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PanCamPlugin)
        .insert_resource(ClearColor(BACKGROUND))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .run();
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component)]
struct SpriteIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default()).insert(PanCam::default());

    let mut rng = rand::thread_rng();
    let perlin = Perlin::new(rng.gen());

    let texture = asset_server.load(SPRITE_SHEET_PATH);
    let layout =
        TextureAtlasLayout::from_grid(
            Vec2::new(TILE_W, TILE_H), 7, 1, None, None
        );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let animation_indices = AnimationIndices { first: 2, last: 3 };

    commands.spawn((
        SpriteSheetBundle {
            texture: texture.clone(),
            atlas: TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_indices.first,
            },
            transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));

    let mut tiles = HashSet::new();
    for x in 0..GRID_COLS {
        for y in 0..GRID_ROWS {
            let val = perlin.get([x as f64 / NOISE_SCALE, y as f64 / NOISE_SCALE]);
            if val < 0.2 {
                continue;
            }

            tiles.insert((x as i32, y as i32));
        }
    }

    for (x, y) in tiles.iter() {
        let (tile, nei_count) = get_tile((*x, *y), &tiles);
        let (x, y) = grid_to_world(*x as f32, *y as f32);

        if nei_count <= 1 {
            continue;
        }

        commands.spawn((
            SpriteSheetBundle {
                sprite: Sprite {
                    color: SAND,
                    ..default()
                },
                texture: texture.clone(),
                atlas: TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: tile,
                },
                transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR))
                    .with_translation(Vec3::new(x, y, 0.0)),
                ..default()
            },
        ));
    }
}

fn get_tile((x, y): (i32, i32), occupied: &HashSet<(i32, i32)>) -> (usize, usize) {
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

fn grid_to_world(x: f32, y: f32) -> (f32, f32) {
    (
        x * TILE_W as f32 * SPRITE_SCALE_FACTOR,
        y * TILE_H as f32 * SPRITE_SCALE_FACTOR,
    )
}

