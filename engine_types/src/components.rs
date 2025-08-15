use std::ops::Mul;

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct GLTFAsset {
    pub path: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Transform {
    pub position: glam::Vec3,
    pub scale: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::IDENTITY,
        }
    }
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
