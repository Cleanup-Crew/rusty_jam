use crate::Collider;
use bevy::prelude::*;
use std::collections::HashMap;

pub const TILE_SIZE: f32 = 64.;

#[derive(Eq, PartialEq, Hash)]
pub enum RoomKind {
    Security,
    Empty,
    Hallway(HallwayKind),
}

#[derive(Eq, PartialEq, Hash)]
pub enum HallwayKind {
    NorthEastSouthWest,
    NorthEastSouth,
    NorthEastWest,
    NorthSouthWest,
    EastSouthWest,
    NorthEast,
    EastSouth,
    SouthWest,
    NorthWest,
    NorthSouth,
    EastWest,
}

pub struct Room {
    pub asset: Handle<ColorMaterial>,
    pub width: usize,
    pub height: usize,
    pub rotation: f32,
    pub colliders: Vec<Collider>,
}

impl Room {
    pub fn spawn(&self, commands: &mut Commands, x: usize, y: usize) {
        // convert map coord to bevy coord
        let x = x as f32 * TILE_SIZE + self.width as f32 * TILE_SIZE / 2.;
        let y = y as f32 * TILE_SIZE + self.height as f32 * TILE_SIZE / 2.;

        let mut transform = Transform::from_xyz(x, y, 0.);
        transform.rotate(Quat::from_rotation_z(self.rotation));
        let mut entity_commands = commands.spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(
                TILE_SIZE * self.width as f32,
                TILE_SIZE * self.height as f32,
            )),
            material: self.asset.clone_weak(),
            transform,
            ..Default::default()
        });

        self.colliders.iter().for_each(|c| {
            entity_commands.with_children(|parent| {
                parent
                    .spawn()
                    .insert(Transform::identity())
                    .insert(GlobalTransform::identity())
                    .insert(c.clone());
            });
        });
    }
}

pub fn load_rooms(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rooms: ResMut<HashMap<RoomKind, Room>>,
) {
    rooms.insert(
        RoomKind::Security,
        Room {
            asset: materials.add(asset_server.load("rooms/security.png").into()),
            width: 3,
            height: 3,
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
        },
    );
    rooms.insert(
        RoomKind::Empty,
        Room {
            asset: materials.add(asset_server.load("rooms/empty.png").into()),
            width: 4,
            height: 3,
            rotation: 0.,
            colliders: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthSouth),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/straight.png").into()),
            width: 1,
            height: 1,
            rotation: 0.,
            colliders: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::EastWest),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/straight.png").into()),
            width: 1,
            height: 1,
            rotation: 0.5 * std::f32::consts::PI,
            colliders: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthEast),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
            width: 1,
            height: 1,
            rotation: 0.,
            colliders: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::EastSouth),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
            width: 1,
            height: 1,
            rotation: 0.5 * std::f32::consts::PI,
            colliders: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::SouthWest),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
            width: 1,
            height: 1,
            rotation: 1. * std::f32::consts::PI,
            colliders: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthWest),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
            width: 1,
            height: 1,
            rotation: 1.5 * std::f32::consts::PI,
            colliders: vec![],
        },
    );
}