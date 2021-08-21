use bevy::prelude::*;

pub struct Room {
    pub asset: Handle<ColorMaterial>,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}

impl Room {
    pub fn spawn(&self, commands: &mut Commands, x: f32, y: f32) {
        let mut transform = Transform::from_xyz(x, y, 0.);
        transform.rotate(Quat::from_rotation_z(self.rotation));
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(self.width, self.height)),
            material: self.asset.clone_weak(),
            transform,
            ..Default::default()
        });
    }
}
