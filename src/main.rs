mod room;
use room::*;

use bevy::{core::FixedTimestep, prelude::*};

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
                .with_system(move_player),
        )
        .add_system(bevy::input::system::exit_on_esc_system)
        .run()
}

struct Player;

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(16., 16.)),
            material: materials.add(Color::GOLD.into()),
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        })
        .insert(Player);
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
    };
    let hallway_straight = Room {
        asset: materials.add(asset_server.load("rooms/hallways/straight.png").into()),
        width: 64.,
        height: 64.,
        rotation: 0.,
    };
    let hallway_straight_90 = Room {
        asset: materials.add(asset_server.load("rooms/hallways/straight.png").into()),
        width: 64.,
        height: 64.,
        rotation: 0.5 * std::f32::consts::PI,
    };
    let hallway_angle = Room {
        asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
        width: 64.,
        height: 64.,
        rotation: 0.,
    };
    let hallway_angle_90 = Room {
        asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
        width: 64.,
        height: 64.,
        rotation: 0.5 * std::f32::consts::PI,
    };
    let hallway_angle_180 = Room {
        asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
        width: 64.,
        height: 64.,
        rotation: 1. * std::f32::consts::PI,
    };
    let hallway_angle_270 = Room {
        asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
        width: 64.,
        height: 64.,
        rotation: 1.5 * std::f32::consts::PI,
    };

    security_room.spawn(&mut commands, 0., 0.);
    hallway_straight_90.spawn(&mut commands, -128., 0.);
    hallway_angle.spawn(&mut commands, -192., 0.);

    rooms.push(security_room);
    rooms.push(hallway_straight_90);
    rooms.push(hallway_angle);

    // desk
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(90., 100.)),
        material: materials.add(asset_server.load("furniture/Security Desk.png").into()),
        transform: Transform::from_xyz(0., 64., 0.),
        ..Default::default()
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
