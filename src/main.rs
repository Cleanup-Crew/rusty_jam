mod mapgen;
mod room;

use bevy::{
    core::{FixedTimestep, FloatOrd},
    prelude::*,
    render::{
        mesh::Indices,
        pipeline::{PipelineDescriptor, RenderPipeline},
        shader::{ShaderStage, ShaderStages},
    },
    sprite,
};
use itertools::Itertools;
use mapgen::*;
use room::*;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(HashMap::<RoomKind, Room>::new())
        .add_startup_system(setup)
        .add_startup_system(room::load_rooms.label("load_rooms"))
        .add_startup_system(generate_world.after("load_rooms"))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1. / 60.))
                .with_system(field_of_vision)
                .with_system(move_player.before("collision"))
                .with_system(collision.label("collision"))
                .with_system(move_camera.after("collision")),
        )
        .add_system(bevy::input::system::exit_on_esc_system)
        .run()
}

struct Player;
struct MainCamera;

#[derive(Copy, Clone)]
pub struct Collider {
    size: Vec2,
    offset: Vec3,
}

impl Collider {
    pub fn new(size: Vec2, offset: Vec2) -> Self {
        Self {
            size,
            offset: (offset, 0.).into(),
        }
    }
}

struct Nonstatic;

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(16., 16.)),
            material: materials.add(Color::GOLD.into()),
            transform: Transform::from_xyz(10. * TILE_SIZE, 10. * TILE_SIZE, 1.),
            ..Default::default()
        })
        .insert(Player)
        .insert(Collider::new(Vec2::new(16., 16.), Vec2::new(0., 0.)))
        .insert(Nonstatic);
}

fn generate_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rooms: Res<HashMap<RoomKind, Room>>,
) {
    // Randomize map
    let mut map = Map::new(20, 20);
    map.generate(&rooms);

    // Spawn entities for map
    for room_kind in map.rooms {
        rooms[&room_kind.0].spawn(&mut commands, room_kind.1 .0, room_kind.1 .1)
    }

    for (hallway_x, hallway_y) in map.hallways {
        rooms[&RoomKind::Hallway(map.occupied.hallway_kind(hallway_x, hallway_y))].spawn(
            &mut commands,
            hallway_x,
            hallway_y,
        );
    }

    // desk
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(90., 100.)),
            material: materials.add(asset_server.load("furniture/Security Desk.png").into()),
            transform: Transform::from_xyz(0., 64., 0.),
            ..Default::default()
        })
        .insert(Collider::new(Vec2::new(90., 58.), Vec2::new(0., -21.)));
    // doors
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(60., 80.)),
        material: materials.add(asset_server.load("furniture/Door_open.png").into()),
        transform: Transform::from_xyz(128., 64., 0.),
        ..Default::default()
    });
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(60., 80.)),
        material: materials.add(asset_server.load("furniture/Door_closed.png").into()),
        transform: Transform::from_xyz(192., 64., 0.),
        ..Default::default()
    });
}

fn field_of_vision(
    mut commands: Commands,
    player_query: Query<&GlobalTransform, With<Player>>,
    query: Query<(&GlobalTransform, &Collider), Without<Player>>,
    input: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // A pipeline will be added with custom shaders
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    // Access to add new shaders
    mut shaders: ResMut<Assets<Shader>>,
) {
    if !input.just_pressed(KeyCode::Y) {
        return;
    }
    // TODO: make this a resource or something
    const RADIUS: f32 = TILE_SIZE * 0.1;

    let player = match player_query.single() {
        Ok(p) => p.translation,
        Err(e) => {
            error!("Player entity not found: {}", e);
            return;
        }
    };

    let mut edges: Vec<(Vec2, Vec2)> = Vec::new();
    let mut rays: Vec<Vec2> = Vec::new();
    for (transform, collider) in query.iter() {
        // get vertices
        let mid = transform.translation + collider.offset;
        let half_size = collider.size / 2.;
        let top = mid.y + half_size.y;
        let bottom = mid.y - half_size.y;
        let right = mid.x + half_size.x;
        let left = mid.x - half_size.x;

        let verts = [
            (left, bottom).into(),
            (left, top).into(),
            (right, top).into(),
            (right, bottom).into(),
        ];
        for vi in 0..4 {
            edges.push((verts[vi], verts[(vi + 1) % 4]));
        }

        let angles = verts
            .iter()
            .map(|&v| (v.y - player.y).atan2(v.x - player.x))
            .collect_vec();

        for ang in angles.iter() {
            let rdx = RADIUS * ang.cos();
            let rdy = RADIUS * ang.sin();

            rays.push((rdx, rdy).into());
            let rdx = RADIUS * ang.cos() + 0.0001;
            let rdy = RADIUS * ang.sin() + 0.0001;

            rays.push((rdx, rdy).into());
            let rdx = RADIUS * ang.cos() - 0.0001;
            let rdy = RADIUS * ang.sin() - 0.0001;

            rays.push((rdx, rdy).into());
        }
    }

    let mut hits = Vec::new();
    for ray in rays.iter() {
        let mut min_t1 = f32::INFINITY;
        let mut min_px = 0.;
        let mut min_py = 0.;
        let mut b_valid = false;

        for &(vert_a, vert_b) in edges.iter() {
            let edge_vec = vert_b - vert_a;
            if (edge_vec.x - ray.x).abs() > 0. && (edge_vec.y - ray.y).abs() > 0. {
                let t2 = (ray.x * (vert_a.y - player.y) + (ray.y * (player.x - vert_a.x)))
                    / (edge_vec.x * ray.y - edge_vec.y * ray.x);
                let t1 = (vert_a.x + edge_vec.x * t2 - player.x) / ray.x;

                if t1 > 0. && t2 >= 0. && t2 <= 1. {
                    if t1 < min_t1 {
                        min_t1 = t1;
                        min_px = player.x + ray.x * t1;
                        min_py = player.y + ray.y * t1;
                        b_valid = true;
                    }
                }
            }
        }

        if b_valid {
            let ang = (min_py - player.y).atan2(min_px - player.x);
            hits.push((ang, min_px, min_py));
        }
    }

    hits.sort_by(|&(ang1, _, _), &(ang2, _, _)| {
        //ang1.partial_cmp(ang2).unwrap_or(std::cmp::Ordering::Equal)
        FloatOrd(ang1).cmp(&FloatOrd(ang2))
    });
    hits.dedup();

    let mut verts = Vec::with_capacity(hits.len() + 1);
    verts.push([player.x, player.y, 0.]);
    for hit in hits.iter() {
        verts.push([hit.1, hit.2, 0.]);
    }

    let mut tris = Vec::new();
    for i in 1..=hits.len() as u32 {
        tris.push(0);
        tris.push(i);
        tris.push((i % hits.len() as u32) + 1);
    }

    let mut v_color = Vec::new();
    for _ in 0..verts.len() {
        v_color.push([1.0, 1.0, 1.0]);
    }

    let mut mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.set_indices(Some(Indices::U32(tris)));
    mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        // Vertex shaders are run once for every vertex in the mesh.
        // Each vertex can have attributes associated to it (e.g. position,
        // color, texture mapping). The output of a shader is per-vertex.
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        // Fragment shaders are run for each pixel belonging to a triangle on
        // the screen. Their output is per-pixel.
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    commands.spawn_bundle(MeshBundle {
        mesh: meshes.add(mesh),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle,
        )]),
        transform: Transform::from_xyz(0., 0., 1.),
        ..Default::default()
    });
}

const VERTEX_SHADER: &str = r"
#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Color;

layout(location = 1) out vec3 v_Color;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    v_Color = Vertex_Color;
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
";

const FRAGMENT_SHADER: &str = r"
#version 450

layout(location = 1) in vec3 v_Color;

layout(location = 0) out vec4 o_Target;

void main() {
    o_Target = vec4(v_Color, 1.0);
}
";

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

fn move_camera(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    player_query: Query<&GlobalTransform, With<Player>>,
) {
    let player = match player_query.single() {
        Ok(p) => p,
        Err(e) => {
            error!("Player entity not found: {}", e);
            return;
        }
    };

    let mut camera = match camera_query.single_mut() {
        Ok(t) => t,
        Err(e) => {
            error!("Main Camera not found: {}", e);
            return;
        }
    };

    camera.translation = player.translation;
}
