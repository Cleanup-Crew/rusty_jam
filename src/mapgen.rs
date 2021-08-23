use crate::room::*;
use pathfinding::prelude::*;
use rand;
use rand::Rng;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileKind {
    Empty,
    Room(usize),
    Door,
    Hallway,
}

impl TileKind {
    fn connects(&self, other: &TileKind) -> bool {
        match *self {
            TileKind::Empty => false,
            TileKind::Room(id) => *other == TileKind::Room(id) || *other == TileKind::Door,
            TileKind::Door => *other != TileKind::Empty,
            TileKind::Hallway => *other == TileKind::Hallway || *other == TileKind::Door,
        }
    }

    fn connects_hallway_pathing(&self, other: &TileKind) -> bool {
        match *self {
            TileKind::Empty => true,
            TileKind::Hallway => *other == TileKind::Hallway || *other == TileKind::Empty,
            _ => false,
        }
    }
}

pub struct TileArray {
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

    pub fn hallway_kind(&self, x: usize, y: usize) -> HallwayKind {
        // Choose cardinal neighbors that aren't out of bounds
        let neighbors = vec![
            (Some(x), Some(y + 1)),
            (Some(x + 1), Some(y)),
            (Some(x), y.checked_sub(1)),
            (x.checked_sub(1), Some(y)),
        ]
        .into_iter()
        .map(|(x, y)| {
            if let Some(x) = x {
                if let Some(y) = y {
                    if x < self.width && y < self.height {
                        if self[(x, y)] == TileKind::Hallway || self[(x, y)] == TileKind::Door {
                            return true;
                        }
                    }
                }
            }
            false
        })
        .collect::<Vec<_>>();

        // (North, East, South, West)
        use HallwayKind::*;
        match (neighbors[0], neighbors[1], neighbors[2], neighbors[3]) {
            (true, true, true, true) => NorthEastSouthWest,
            (true, true, true, false) => NorthEastSouth,
            (true, true, false, true) => NorthEastWest,
            (true, false, true, true) => NorthSouthWest,
            (false, true, true, true) => EastSouthWest,
            (true, true, false, false) => NorthEast,
            (true, false, true, false) => NorthSouth,
            (true, false, false, true) => NorthWest,
            (false, true, true, false) => EastSouth,
            (false, true, false, true) => EastWest,
            (false, false, true, true) => SouthWest,
            (true, false, false, false) => North,
            (false, true, false, false) => East,
            (false, false, true, false) => South,
            (false, false, false, true) => West,
            (false, false, false, false) => NorthEastSouthWest,
        }
    }

    fn get_connections<F>(
        &self,
        x: usize,
        y: usize,
        connectivity: F,
    ) -> Vec<(usize, usize, TileKind)>
    where
        F: Fn(&TileKind, &TileKind) -> bool,
    {
        {
            let tile = self[(x, y)];
            let mut adj = Vec::with_capacity(4);

            // Choose cardinal neighbors that aren't out of bounds
            vec![
                (x.checked_sub(1), Some(y)),
                (Some(x + 1), Some(y)),
                (Some(x), y.checked_sub(1)),
                (Some(x), Some(y + 1)),
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
            // Check for connectivity to neighbors
            .for_each(|(x, y)| {
                let neighbor_type = self[(x, y)];
                if connectivity(&neighbor_type, &tile) {
                    adj.push((x, y, neighbor_type));
                }
            });

            adj
        }
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
    pub occupied: TileArray,
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
        let mut connected_hallways = self.connected_hallways();

        while connected_hallways.len() > 1 {
            //Connect set 0 and 1

            let path = bfs(
                &connected_hallways[0][0],
                |&(x, y, _)| {
                    self.occupied
                        .get_connections(x, y, TileKind::connects_hallway_pathing)
                },
                |&t| {
                    t == connected_hallways[1][0]
                        || (t.2 == TileKind::Hallway && !connected_hallways[0].contains(&t))
                },
            );

            if let Some(path) = path {
                for (x, y, _) in path {
                    self.occupied[(x, y)] = TileKind::Hallway;
                    self.hallways.push((x, y));
                }
            } else {
                panic!("Unconnectable hallway in mapgen");
            }
            connected_hallways = self.connected_hallways();
        }

        // connect hallways with <2 neighbors to nearest hallway
        let mut singles = self.get_single_hallways();
        while singles.len() > 0 {
            let path = bfs(
                &singles[0],
                |&(x, y, _)| {
                    self.occupied
                        .get_connections(x, y, TileKind::connects_hallway_pathing)
                },
                |&t| t.2 == TileKind::Hallway && t != singles[0],
            );

            if let Some(path) = path {
                for (x, y, _) in path {
                    self.occupied[(x, y)] = TileKind::Hallway;
                    self.hallways.push((x, y));
                }
            } else {
                panic!("Unconnectable hallway in mapgen");
            }
            singles = self.get_single_hallways();
        }
    }

    fn get_single_hallways(&self) -> Vec<(usize, usize, TileKind)> {
        self.hallways
            .iter()
            .filter_map(|(x, y)| match self.occupied.hallway_kind(*x, *y) {
                HallwayKind::North | HallwayKind::East | HallwayKind::South | HallwayKind::West => {
                    Some((*x, *y, TileKind::Hallway))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    fn connected_hallways(&self) -> Vec<Vec<(usize, usize, TileKind)>> {
        let components = connected_components(
            &self
                .occupied
                .iter()
                .filter(|&(_, _, tile)| tile != TileKind::Empty)
                .collect::<Vec<_>>(),
            |&(x, y, _)| self.occupied.get_connections(x, y, TileKind::connects),
        );

        // filter for Hallways
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
