use egui::*;

use crate::block_registry::Voxel;

type U3 = (usize, usize, usize);


#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroundDirection {
    XMinus,
    ZMinus,
    XPlus,
    ZPlus,
}

impl GroundDirection {
    pub fn turn_left(&mut self) {
        match self {
            Self::XMinus => { *self = Self::ZMinus; }
            Self::ZMinus => { *self = Self::XPlus; }
            Self::XPlus => { *self = Self::ZPlus; }
            Self::ZPlus => { *self = Self::XMinus; }
        }
    }
    
    pub fn turn_right(&mut self) {
        match self {
            Self::XMinus => { *self = Self::ZPlus; }
            Self::ZMinus => { *self = Self::XMinus; }
            Self::XPlus => { *self = Self::ZMinus; }
            Self::ZPlus => { *self = Self::XPlus; }
        }
    }
    
    pub fn turn_around(&mut self) {
        match self {
            Self::XMinus => { *self = Self::XPlus; }
            Self::ZMinus => { *self = Self::ZPlus; }
            Self::XPlus => { *self = Self::XMinus; }
            Self::ZPlus => { *self = Self::ZMinus; }
        }
    }
}



#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Ground(GroundDirection),
    Up,
    Down,
}

impl Direction {
    pub fn offset(self, pos: U3) -> Option<U3> {
        let (x, y, z) = pos;
        Some(match self {
            Self::Ground(GroundDirection::XMinus) => (x.checked_sub(1)?, y, z),
            Self::Ground(GroundDirection::ZMinus) => (x, y, z.checked_sub(1)?),
            Self::Ground(GroundDirection::XPlus)  => (x.checked_add(1)?, y, z),
            Self::Ground(GroundDirection::ZPlus)  => (x, y, z.checked_add(1)?),
            Self::Up   => (x, y.checked_add(1)?, z),
            Self::Down => (x, y.checked_sub(1)?, z),
        })
    }
}


pub struct Chunk<const S: usize> {
    pub blocks: Box<[[[Voxel; S]; S]; S]>,
    pub robot_cut: U3,
    pub robot_dir: GroundDirection,
}

impl<const S: usize> Chunk<S> {
    fn new() -> Self {
        let robot_cut = (S - 1, (S - 1) / 2, S - 1);
        Self {
            blocks: Voxel::summon_grid(robot_cut),
            robot_cut,
            robot_dir: GroundDirection::ZPlus,
        }
    }
    
    pub fn robot_faces(
        &mut self,
        override_direction: Option<Direction>
    ) -> Option<(&mut Voxel, U3)> {
        let (x, y, z) = override_direction
            .unwrap_or(Direction::Ground(self.robot_dir))
            .offset(self.robot_cut)?;
        Some((
            self.blocks.get_mut(z)?.get_mut(y)?.get_mut(x)?,
            (x, y, z)
        ))
    }
    
    pub fn robot_detect(&mut self, d: Option<Direction>) -> Option<bool> {
        let swung_block = self.robot_faces(d)?.0;
        Some(swung_block.is_tough())
    }
    
    pub fn robot_swing(&mut self, d: Option<Direction>) -> Option<Voxel> {
        let swung_block = self.robot_faces(d)?.0;
        let material = std::mem::replace(swung_block, Voxel::Air);
        Some(material)
    }
    
    pub fn robot_move(&mut self, d: Option<Direction>) -> bool {
        match self.robot_faces(d) {
            Some((Voxel::Air, pos)) => { self.robot_cut = pos; true },
            _ => false,
        }
    }
    
    fn draw_callback(&self, mut painter: impl FnMut(f32,f32,f32,bool,&Voxel)) {
        for z in 0..S {
            for y in 0..S {
                for x in 0..S {
                    let (x0, y0, z0) = self.robot_cut;
                    let transparent = (x0 < x) | (y0 < y) | (z0 < z);
                    
                    painter(x as f32, y as f32, z as f32, transparent,
                            &self.blocks[z][y][x]);
                }
            }
        }
    }
}

impl<const S: usize> Default for Chunk<S> {
    fn default() -> Self { Self::new() }
}

impl<const S: usize> Widget for &Chunk<S> {
    fn ui(self, ui: &mut Ui) -> Response {
        const HEX: f32 = 0.8660254;
        const IHEX: f32 = 1.1547005;
        
        // 1. Space requirements.
        let s = S as f32;
        
        let cell = (ui.available_width() * IHEX / s)
               .min(ui.available_height() / s);
        let grid = vec2(cell * s * HEX, cell * s);
        let sense = Sense::empty();
        
        let u = vec2(0.0, -cell / 2.0);
        let v = vec2(-cell * HEX / 2.0, cell / 4.0);
        let w = vec2(cell * HEX / 2.0, cell / 4.0);
        
        // 2. Space.
        let (response, painter) = ui.allocate_painter(grid, sense);
        let rect = response.rect;
        let center = rect.center();
        
        // 3. Drawing.
        self.draw_callback(|x, y, z, transparent, voxel| {
            let Some(mut color) = crate::block_colors::c(voxel) else { return };
            if transparent {
                color = color.gamma_multiply(0.13);
            }
            
            let back_corner = center + v * x + u * y + w * z;
            let mesh = epaint::PathShape::convex_polygon(
                vec![back_corner + u, back_corner + u + w,
                     back_corner + w, back_corner - u,
                     back_corner + v, back_corner + v + u],
                color,
                epaint::PathStroke::NONE
            );
            
            painter.add(mesh);
            if !transparent {
                painter.add(epaint::PathShape::convex_polygon(
                    vec![back_corner, back_corner + w + u,
                         back_corner + w, back_corner - u],
                    Color32::from_black_alpha(12),
                    epaint::PathStroke::NONE
                ));
                painter.add(epaint::PathShape::convex_polygon(
                    vec![back_corner, back_corner - u,
                         back_corner + v, back_corner + u + v],
                    Color32::from_white_alpha(7),
                    epaint::PathStroke::NONE
                ));
            }
        });
        
        let (rx, ry, rz) = self.robot_cut;
        let robot_center = center + v * (rx as f32) + u * (ry as f32) + w * (rz as f32);
        painter.circle_filled(robot_center, cell / 4.0, Color32::BLUE);
        
        response
    }
}
