use crate::Collider;
use bevy::prelude::*;

pub struct Room {
    pub asset: Handle<ColorMaterial>,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub colliders: Vec<Collider>,
}

impl Room {
    pub fn spawn(&self, commands: &mut Commands, x: f32, y: f32) {
        let mut transform = Transform::from_xyz(x, y, 0.);
        transform.rotate(Quat::from_rotation_z(self.rotation));
        let mut entity_commands = commands.spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(self.width, self.height)),
            material: self.asset.clone_weak(),
            transform,
            ..Default::default()
        });

        self.colliders.iter().for_each(|c| {
            entity_commands.with_children(|parent| {
                parent
                    .spawn()
                    .insert(Transform::identity())
                    .insert(c.clone());
            });
        });
    }
}
