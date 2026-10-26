use crate::block_registry::Voxel;
use egui::Color32;

#[allow(unreachable_patterns)]
pub fn c(v: &Voxel) -> Option<Color32> {
    match v {
        Voxel::Rocks => Some(Color32::GRAY),
        Voxel::Ore => Some(Color32::GOLD),
        Voxel::Air => None,
        _ => Some(Color32::PLACEHOLDER),
    }
}

