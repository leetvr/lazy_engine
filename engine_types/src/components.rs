use std::ops::Mul;

use glam::EulerRot;
use serde::{Deserialize, Serialize};
use yakui::label;

use crate::CanYak;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GLTFAsset {
    pub path: String,
}

impl CanYak for GLTFAsset {
    fn get_paint_fn() -> crate::PaintFn {
        Box::new(|world, entity| {
            let asset = world.get::<&GLTFAsset>(entity).unwrap();
            label(asset.path.clone());
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Transform {
    #[serde(default)]
    pub position: glam::Vec3,
    #[serde(default = "default_scale")]
    pub scale: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            scale: default_scale(),
            rotation: glam::Quat::IDENTITY,
        }
    }
}

impl CanYak for Transform {
    fn get_paint_fn() -> crate::PaintFn {
        Box::new(|world, entity| {
            let transform = world.get::<&Transform>(entity).unwrap();
            label(format!("Position: {}", transform.position));
            label(format!("Rotation: {}", pretty_rotation(transform.rotation)));
            label(format!("Scale: {}", transform.scale));
        })
    }
}

fn pretty_rotation(rotation: glam::Quat) -> String {
    let (y, x, z) = rotation.to_euler(EulerRot::YXZ);
    format!(
        "x: {:.2}, y: {:.2}, z: {:.2}",
        x.to_degrees(),
        y.to_degrees(),
        z.to_degrees()
    )
}

const fn default_scale() -> glam::Vec3 {
    glam::Vec3::ONE
}

impl Transform {
    fn to_affine(&self) -> glam::Affine3A {
        glam::Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

impl Mul<&Transform> for glam::Mat4 {
    type Output = glam::Mat4;

    fn mul(self, transform: &Transform) -> Self::Output {
        self * transform.to_affine()
    }
}
