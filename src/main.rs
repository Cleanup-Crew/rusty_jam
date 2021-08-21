mod room;
use room::*;

use bevy::{core::FixedTimestep, prelude::*, sprite};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLUE))
        .insert_resource(Vec::<Room>::new())
        .add_startup_system(setup)
        .add_startup_system(generate_world)
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
    mut rooms: ResMut<Vec<Room>>,
) {
    // load assets
    let security_room = Room {
        asset: materials.add(asset_server.load("rooms/security.png").into()),
        width: 192.,
        height: 192.,
        rotation: 0.,
        colliders: vec![
            Collider {
                size: Vec2::new(192., 16.),
                offset: Vec3::new(0., -88., 0.),
            },
            Collider {
                size: Vec2::new(192., 16.),
                offset: Vec3::new(0., 88., 0.),
            },
            Collider {
                size: Vec2::new(16., 64.),
                offset: Vec3::new(-88., -48., 0.),
            },
            Collider {
                size: Vec2::new(16., 64.),
                offset: Vec3::new(-88., 48., 0.),
            },
            Collider {
                size: Vec2::new(16., 64.),
                offset: Vec3::new(88., -48., 0.),
            },
            Collider {
                size: Vec2::new(16., 64.),
                offset: Vec3::new(88., 48., 0.),
            },
        ],
    };
    let empty_room = Room {
        asset: materials.add(asset_server.load("rooms/empty.png").into()),
        width: 256.,
        height: 192.,
        rotation: 0.,
        colliders: vec![],
    };
    let hallway_straight = Room {
        asset: materials.add(asset_server.load("rooms/hallways/straight.png").into()),
        width: 64.,
        height: 64.,
        rotation: 0.,
        colliders: vec![],
    };
    let hallway_straight_90 = Room {
        asset: materials.add(asset_server.load("rooms/hallways/straight.png").into()),
        width: 64.,
        height: 64.,
        rotation: 0.5 * std::f32::consts::PI,
        colliders: vec![],
    };
    let hallway_angle = Room {
        asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
        width: 64.,
        height: 64.,
        rotation: 0.,
        colliders: vec![],
    };
    let hallway_angle_90 = Room {
        asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
        width: 64.,
        height: 64.,
        rotation: 0.5 * std::f32::consts::PI,
        colliders: vec![],
    };
    let hallway_angle_180 = Room {
        asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
        width: 64.,
        height: 64.,
        rotation: 1. * std::f32::consts::PI,
        colliders: vec![],
    };
    let hallway_angle_270 = Room {
        asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
        width: 64.,
        height: 64.,
        rotation: 1.5 * std::f32::consts::PI,
        colliders: vec![],
    };

    security_room.spawn(&mut commands, 0., 0.);
    hallway_straight_90.spawn(&mut commands, -128., 0.);
    hallway_angle.spawn(&mut commands, -192., 0.);
    hallway_straight.spawn(&mut commands, -192., 64.);
    empty_room.spawn(&mut commands, -160., 192.);

    rooms.push(security_room);
    rooms.push(hallway_straight);
    rooms.push(hallway_straight_90);
    rooms.push(hallway_angle);
    rooms.push(empty_room);

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

fn collision(
    mut q0: Query<(&mut Transform, &Collider), With<Nonstatic>>,
    q1: Query<(&Transform, &Collider), Without<Nonstatic>>,
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
