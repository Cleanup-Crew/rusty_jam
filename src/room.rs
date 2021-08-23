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
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

pub struct Room {
    pub asset: Handle<ColorMaterial>,
    pub width: usize,
    pub height: usize,
    pub rotation: f32,
    pub colliders: Vec<Collider>,
    pub doors: Vec<(usize, usize, Direction)>,
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
            doors: vec![(0, 1, Direction::West), (2, 1, Direction::East)],
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
            doors: vec![(1, 0, Direction::South), (0, 1, Direction::West)],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::North),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/one.png").into()),
            width: 1,
            height: 1,
            rotation: 0.,
            colliders: vec![],
            doors: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::West),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/one.png").into()),
            width: 1,
            height: 1,
            rotation: 0.5 * std::f32::consts::PI,
            colliders: vec![],
            doors: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::South),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/one.png").into()),
            width: 1,
            height: 1,
            rotation: 1. * std::f32::consts::PI,
            colliders: vec![],
            doors: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::East),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/one.png").into()),
            width: 1,
            height: 1,
            rotation: 1.5 * std::f32::consts::PI,
            colliders: vec![],
            doors: vec![],
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
            doors: vec![],
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
            doors: vec![],
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
            doors: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthWest),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
            width: 1,
            height: 1,
            rotation: 0.5 * std::f32::consts::PI,
            colliders: vec![],
            doors: vec![],
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
            doors: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::EastSouth),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/angle.png").into()),
            width: 1,
            height: 1,
            rotation: 1.5 * std::f32::consts::PI,
            colliders: vec![],
            doors: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthEastWest),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/tee.png").into()),
            width: 1,
            height: 1,
            rotation: 0.,
            colliders: vec![],
            doors: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthSouthWest),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/tee.png").into()),
            width: 1,
            height: 1,
            rotation: 0.5 * std::f32::consts::PI,
            colliders: vec![],
            doors: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::EastSouthWest),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/tee.png").into()),
            width: 1,
            height: 1,
            rotation: 1.0 * std::f32::consts::PI,
            colliders: vec![],
            doors: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthEastSouth),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/tee.png").into()),
            width: 1,
            height: 1,
            rotation: 1.5 * std::f32::consts::PI,
            colliders: vec![],
            doors: vec![],
        },
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthEastSouthWest),
        Room {
            asset: materials.add(asset_server.load("rooms/hallways/four.png").into()),
            width: 1,
            height: 1,
            rotation: 0.,
            colliders: vec![],
            doors: vec![],
        },
    );
}
