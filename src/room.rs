use crate::Collider;
use bevy::prelude::*;
use itertools::Itertools;
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
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
    pub fn new(
        asset: Handle<ColorMaterial>,
        width: usize,
        height: usize,
        doors: Vec<(usize, Direction)>,
    ) -> Self {
        let coll_width = 16.;
        let coll_height = 16.;
        let door_size = 32.;
        let door_wall = (TILE_SIZE - door_size) / 2.;
        let width_f = width as f32 * TILE_SIZE;
        let height_f = height as f32 * TILE_SIZE;
        let offset_x = width_f / 2. - coll_width / 2.;
        let offset_y = height_f / 2. - coll_height / 2.;

        let mut door_map = HashMap::new();
        door_map.insert(Direction::North, Vec::new());
        door_map.insert(Direction::East, Vec::new());
        door_map.insert(Direction::South, Vec::new());
        door_map.insert(Direction::West, Vec::new());
        doors
            .iter()
            .for_each(|&(offset, dir)| door_map.get_mut(&dir).unwrap().push(offset));
        // sort vecs
        for vec in door_map.values_mut() {
            vec.sort();
        }

        // redefine doors
        let mut doors = Vec::with_capacity(doors.len());

        //
        let mut colliders = Vec::new();

        // goodluck figuring this out in the future
        if let Some(north_doors) = door_map.get(&Direction::North) {
            let mut points = Vec::new();
            points.push(0.);

            // calculate two points on either side of the door
            north_doors.iter().for_each(|&offset| {
                let off_f = offset as f32 * TILE_SIZE;
                let mid = off_f + door_size;
                points.push(mid - door_wall);
                points.push(mid + door_wall);
                doors.push((offset, height - 1, Direction::North));
            });

            points.push(width_f);

            //construct colliders
            for (p1, p2) in points.into_iter().tuples() {
                // don't gen a collider with a zero-sized dimension
                if p1 == p2 {
                    continue;
                }
                let size = p2 - p1;
                let offset = p1 - width_f / 2. + size / 2.;
                colliders.push(Collider::new(
                    Vec2::new(size, coll_height),
                    Vec2::new(offset, offset_y),
                ));
            }
        }
        if let Some(east_doors) = door_map.get(&Direction::East) {
            let mut points = Vec::new();
            points.push(coll_height);

            // calculate two points on either side of the door
            east_doors.iter().for_each(|&offset| {
                let off_f = offset as f32 * TILE_SIZE;
                let mid = off_f + door_size;
                points.push(mid - door_wall);
                points.push(mid + door_wall);
                doors.push((width - 1, offset, Direction::East));
            });

            points.push(height_f - coll_height);

            //construct colliders
            for (p1, p2) in points.into_iter().tuples() {
                if p1 == p2 {
                    continue;
                }
                let size = p2 - p1;
                let offset = p1 - height_f / 2. + size / 2.;
                colliders.push(Collider::new(
                    Vec2::new(coll_width, size),
                    Vec2::new(offset_x, offset),
                ));
            }
        }
        if let Some(south_doors) = door_map.get(&Direction::South) {
            let mut points = Vec::new();
            points.push(0.);

            // calculate two points on either side of the door
            south_doors.iter().for_each(|&offset| {
                let off_f = offset as f32 * TILE_SIZE;
                let mid = off_f + door_size;
                points.push(mid - door_wall);
                points.push(mid + door_wall);
                doors.push((offset, 0, Direction::South));
            });

            points.push(width_f);

            //construct colliders
            for (p1, p2) in points.into_iter().tuples() {
                if p1 == p2 {
                    continue;
                }
                let size = p2 - p1;
                let offset = p1 - width_f / 2. + size / 2.;
                colliders.push(Collider::new(
                    Vec2::new(size, coll_height),
                    Vec2::new(offset, -offset_y),
                ));
            }
        }
        if let Some(west_doors) = door_map.get(&Direction::West) {
            let mut points = Vec::new();
            points.push(coll_height);

            // calculate two points on either side of the door
            west_doors.iter().for_each(|&offset| {
                let off_f = offset as f32 * TILE_SIZE;
                let mid = off_f + door_size;
                points.push(mid - door_wall);
                points.push(mid + door_wall);
                doors.push((0, offset, Direction::West));
            });

            points.push(height_f - coll_height);

            //construct colliders
            for (p1, p2) in points.into_iter().tuples() {
                if p1 == p2 {
                    continue;
                }
                let size = p2 - p1;
                let offset = p1 - height_f / 2. + size / 2.;
                colliders.push(Collider::new(
                    Vec2::new(coll_width, size),
                    Vec2::new(-offset_x, offset),
                ));
            }
        }

        Self {
            asset,
            width,
            height,
            rotation: 0.,
            colliders,
            doors,
        }
    }
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
    use self::Direction::*;
    rooms.insert(
        RoomKind::Security,
        Room::new(
            materials.add(asset_server.load("rooms/security.png").into()),
            3,
            3,
            vec![(1, Direction::West), (1, Direction::East)],
        ),
    );
    rooms.insert(
        RoomKind::Empty,
        Room::new(
            materials.add(asset_server.load("rooms/empty.png").into()),
            4,
            3,
            vec![(1, Direction::South), (1, Direction::West)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::North),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/one/N.png").into()),
            1,
            1,
            vec![(0, North)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::West),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/one/W.png").into()),
            1,
            1,
            vec![(0, West)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::South),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/one/S.png").into()),
            1,
            1,
            vec![(0, South)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::East),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/one/E.png").into()),
            1,
            1,
            vec![(0, East)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthSouth),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/straight/NS.png").into()),
            1,
            1,
            vec![(0, North), (0, South)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::EastWest),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/straight/EW.png").into()),
            1,
            1,
            vec![(0, East), (0, West)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthEast),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/angle/NE.png").into()),
            1,
            1,
            vec![(0, North), (0, East)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthWest),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/angle/NW.png").into()),
            1,
            1,
            vec![(0, North), (0, West)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::SouthWest),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/angle/SW.png").into()),
            1,
            1,
            vec![(0, South), (0, West)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::EastSouth),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/angle/ES.png").into()),
            1,
            1,
            vec![(0, East), (0, South)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthEastWest),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/tee/NEW.png").into()),
            1,
            1,
            vec![(0, North), (0, East), (0, West)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthSouthWest),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/tee/NSW.png").into()),
            1,
            1,
            vec![(0, North), (0, South), (0, West)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::EastSouthWest),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/tee/ESW.png").into()),
            1,
            1,
            vec![(0, East), (0, South), (0, West)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthEastSouth),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/tee/NES.png").into()),
            1,
            1,
            vec![(0, North), (0, East), (0, South)],
        ),
    );
    rooms.insert(
        RoomKind::Hallway(HallwayKind::NorthEastSouthWest),
        Room::new(
            materials.add(asset_server.load("rooms/hallways/four.png").into()),
            1,
            1,
            vec![(0, North), (0, East), (0, South), (0, West)],
        ),
    );
}
