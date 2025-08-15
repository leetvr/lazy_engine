use std::ops::Mul;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug)]
pub struct GLTFAsset {
    pub path: String,
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
