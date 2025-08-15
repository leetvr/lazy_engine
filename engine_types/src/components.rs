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
