use crate::room::*;
use pathfinding::prelude::*;
use rand;
use rand::Rng;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TileKind {
    Empty,
    Room(usize),
    Door,
    Hallway,
}

impl TileKind {
    fn connects(&self, other: TileKind) -> bool {
        match *self {
            TileKind::Empty => false,
            TileKind::Room(id) => other == TileKind::Room(id) || other == TileKind::Door,
            TileKind::Door => other != TileKind::Empty,
            TileKind::Hallway => other == TileKind::Hallway || other == TileKind::Door,
        }
    }
}

struct TileArray {
    inner: Vec<TileKind>,
    width: usize,
    height: usize,
}

impl TileArray {
    fn new(width: usize, height: usize) -> Self {
        Self {
            inner: vec![TileKind::Empty; width * height],
            width,
            height,
        }
    }

    fn iter(&self) -> impl Iterator<Item = (usize, usize, TileKind)> + '_ {
        self.inner
            .iter()
            .enumerate()
            .map(move |(i, &tile)| (i % self.width, i / self.width, tile))
    }
}

impl Debug for TileArray {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for y in 0..self.height {
            write!(
                fmt,
                "{:?}\n",
                &self.inner[y * self.width..y * self.width + self.width],
            )?;
        }
        Ok(())
    }
}

impl Index<(usize, usize)> for TileArray {
    type Output = TileKind;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        if index.0 >= self.width {
            panic!("X index {} out of bounds", index.0);
        }

        if index.1 >= self.height {
            panic!("Y index {} out of bounds", index.1);
        }

        &self.inner[index.1 * self.width + index.0]
    }
}

impl IndexMut<(usize, usize)> for TileArray {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        if index.0 >= self.width {
            panic!("X index {} out of bounds", index.0);
        }

        if index.1 >= self.height {
            panic!("Y index {} out of bounds", index.1);
        }

        &mut self.inner[index.1 * self.width + index.0]
    }
}
pub struct Map {
    occupied: TileArray,
    pub rooms: Vec<(RoomKind, (usize, usize))>,
    pub hallways: Vec<(usize, usize)>,
    width: usize,
    height: usize,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            occupied: TileArray::new(width, height),
            rooms: vec![],
            hallways: vec![],
            width,
            height,
        }
    }

    pub fn generate(&mut self, rooms: &HashMap<RoomKind, Room>) {
        let mut rng = rand::thread_rng();

        // place security room
        let security_room = &rooms[&RoomKind::Security];
        let (sec_x, sec_y) = (
            self.width / 2 - security_room.width / 2,
            self.height / 2 - security_room.height / 2,
        );

        self.rooms.push((RoomKind::Security, (sec_x, sec_y)));
        for y in sec_y..security_room.height + sec_y {
            for x in sec_x..security_room.width + sec_x {
                self.occupied[(x, y)] = TileKind::Room(0);
            }
        }
        for (dx, dy, _) in room_doors(security_room, sec_x, sec_y) {
            self.occupied[(dx, dy)] = TileKind::Door;
        }

        let hallways = room_adjacent_hallways(security_room, sec_x, sec_y);
        for (x, y) in hallways {
            self.hallways.push((x, y));
            self.occupied[(x, y)] = TileKind::Hallway;
        }

        // place random rooms
        for id in 1..=8 {
            // pick room (only one option for now)
            let room = &rooms[&RoomKind::Empty];

            let room_x = rng.gen_range(1..self.width - room.width);
            let room_y = rng.gen_range(1..self.height - room.height);

            let hallways = room_adjacent_hallways(room, room_x, room_y);

            // Check target positions
            let mut blocked = false;
            for y in room_y..room.height + room_y {
                for x in room_x..room.width + room_x {
                    if self.occupied[(x, y)] != TileKind::Empty {
                        blocked = true;
                    }
                }
            }
            for (x, y) in hallways.clone() {
                if self.occupied[(x, y)] != TileKind::Empty {
                    blocked = true;
                    break;
                }
            }
            if blocked {
                continue;
            }

            // actully place room and hallways
            self.rooms.push((RoomKind::Empty, (room_x, room_y)));
            for y in room_y..room.height + room_y {
                for x in room_x..room.width + room_x {
                    self.occupied[(x, y)] = TileKind::Room(id);
                }
            }
            for (dx, dy, _) in room_doors(room, room_x, room_y) {
                self.occupied[(dx, dy)] = TileKind::Door;
            }

            for (x, y) in hallways {
                self.hallways.push((x, y));
                self.occupied[(x, y)] = TileKind::Hallway;
            }
        }

        // place hallways
        // 1. determine connectivity and create sets
        // 2. connect unconnected sets, thus merging them
        // 3. profit
        let connected_hallways = self.connected_hallways();
        bevy::prelude::info!("{:#?}", connected_hallways);
    }

    fn connected_hallways(&self) -> Vec<Vec<(usize, usize, TileKind)>> {
        let components = connected_components(
            &self
                .occupied
                .iter()
                .filter(|&(_, _, tile)| tile != TileKind::Empty)
                .collect::<Vec<_>>(),
            |&(node_x, node_y, tile)| {
                let mut adj = Vec::with_capacity(4);

                // it's not right
                vec![
                    (node_x.checked_sub(1), Some(node_y)),
                    (Some(node_x + 1), Some(node_y)),
                    (Some(node_x), node_y.checked_sub(1)),
                    (Some(node_x), Some(node_y + 1)),
                ]
                .into_iter()
                .filter_map(|(x, y)| {
                    if let Some(x) = x {
                        if let Some(y) = y {
                            if x < self.width && y < self.height {
                                return Some((x, y));
                            }
                        }
                    }
                    None
                })
                .for_each(|(x, y)| {
                    let neighbor_type = self.occupied[(x, y)];
                    if neighbor_type.connects(tile) {
                        adj.push((x, y, neighbor_type));
                    }
                });

                adj
            },
        );

        components
            .into_iter()
            .map(|set| {
                set.into_iter()
                    .filter(|(_, _, tile)| *tile == TileKind::Hallway)
                    .collect::<Vec<(usize, usize, TileKind)>>()
            })
            .collect::<Vec<_>>()
    }
}

fn room_doors(
    room: &Room,
    room_x: usize,
    room_y: usize,
) -> impl Iterator<Item = (usize, usize, crate::room::Direction)> + Clone + '_ {
    room.doors
        .iter()
        .map(move |(dx, dy, dir)| (dx + room_x, dy + room_y, *dir))
}

fn room_adjacent_hallways(
    room: &Room,
    room_x: usize,
    room_y: usize,
) -> impl Iterator<Item = (usize, usize)> + Clone + '_ {
    room_doors(room, room_x, room_y).map(move |(dx, dy, dir)| match dir {
        crate::room::Direction::North => (dx, dy + 1),
        crate::room::Direction::East => (dx + 1, dy),
        crate::room::Direction::South => (dx, dy - 1),
        crate::room::Direction::West => (dx - 1, dy),
    })
}
