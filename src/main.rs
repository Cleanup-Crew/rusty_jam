mod mapgen;
mod room;

use bevy::{core::FixedTimestep, prelude::*, sprite};
use mapgen::*;
use room::*;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLUE))
        .insert_resource(HashMap::<RoomKind, Room>::new())
        .add_startup_system(setup)
        .add_startup_system(room::load_rooms.label("load_rooms"))
        .add_startup_system(generate_world.after("load_rooms"))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1. / 60.))
                .with_system(move_player.before("collision"))
                .with_system(collision.label("collision")),
        )
        .add_system(bevy::input::system::exit_on_esc_system)
        .run()
}

struct Player;

#[derive(Default, Copy, Clone)]
pub struct Collider {
    size: Vec2,
    offset: Vec3,
}

struct Nonstatic;

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(16., 16.)),
            material: materials.add(Color::GOLD.into()),
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        })
        .insert(Player)
        .insert(Collider {
            size: Vec2::new(16., 16.),
            ..Default::default()
        })
        .insert(Nonstatic);
}

fn generate_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rooms: ResMut<HashMap<RoomKind, Room>>,
) {
    // Randomize map
    let mut map = Map::new(20, 20);
    map.generate(&rooms);

    // Temp (pretend map.generate actually did stuff)
    rooms[&RoomKind::Security].spawn(&mut commands, 0., 0.);
    rooms[&RoomKind::Hallway(HallwayKind::EastWest)].spawn(&mut commands, -128., 0.);
    rooms[&RoomKind::Hallway(HallwayKind::NorthEast)].spawn(&mut commands, -192., 0.);
    rooms[&RoomKind::Hallway(HallwayKind::NorthSouth)].spawn(&mut commands, -192., 64.);
    rooms[&RoomKind::Empty].spawn(&mut commands, -160., 192.);

    // desk
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(90., 100.)),
            material: materials.add(asset_server.load("furniture/Security Desk.png").into()),
            transform: Transform::from_xyz(0., 64., 0.),
            ..Default::default()
        })
        .insert(Collider {
            size: Vec2::new(90., 58.),
            offset: Vec3::new(0., -21., 0.),
        });
}

fn move_player(input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Player>>) {
    const PLAYER_SPEED: f32 = 2.;
    if let Ok(mut player) = query.single_mut() {
        // TODO: Normalize speed
        if input.pressed(KeyCode::W) {
            player.translation.y += PLAYER_SPEED;
        } else if input.pressed(KeyCode::R) {
            player.translation.y -= PLAYER_SPEED;
        }

        if input.pressed(KeyCode::A) {
            player.translation.x -= PLAYER_SPEED;
        } else if input.pressed(KeyCode::S) {
            player.translation.x += PLAYER_SPEED;
        }
    }
}

// FIXME: Assuming the nonstatic transform is relative to the reference frame may limit us in the
// future.
fn collision(
    mut q0: Query<(&mut Transform, &Collider), With<Nonstatic>>,
    q1: Query<(&GlobalTransform, &Collider), Without<Nonstatic>>,
) {
    use sprite::collide_aabb;
    use sprite::collide_aabb::Collision;
    for (mut tran, coll) in q0.iter_mut() {
        for (static_tran, static_coll) in q1.iter() {
            let collision = collide_aabb::collide(
                tran.translation + coll.offset,
                coll.size,
                static_tran.translation + static_coll.offset,
                static_coll.size,
            );

            if let Some(side) = collision {
                match side {
                    Collision::Left => {
                        tran.translation.x = static_tran.translation.x + static_coll.offset.x
                            - (static_coll.size.x / 2.)
                            - (coll.size.x / 2.);
                    }
                    Collision::Right => {
                        tran.translation.x = static_tran.translation.x
                            + static_coll.offset.x
                            + (static_coll.size.x / 2.)
                            + (coll.size.x / 2.);
                    }
                    Collision::Top => {
                        tran.translation.y = static_tran.translation.y
                            + static_coll.offset.y
                            + (static_coll.size.y / 2.)
                            + (coll.size.y / 2.);
                    }
                    Collision::Bottom => {
                        tran.translation.y = static_tran.translation.y + static_coll.offset.y
                            - (static_coll.size.y / 2.)
                            - (coll.size.y / 2.);
                    }
                }
            }
        }
    }
}
