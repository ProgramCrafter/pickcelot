use crate::block_registry::Voxel;
use egui::Color32;

pub fn c(v: &Voxel) -> Option<Color32> {
    match v {
        Voxel::Stone => Some(Color32::LIGHT_GRAY),
        Voxel::Ore => Some(Color32::BROWN),
        Voxel::Air => None,
        _ => Some(Color32::PLACEHOLDER),
    }
}

