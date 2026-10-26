use std::collections::HashMap;
use anyhow::anyhow;
use egui::Color32;

use crate::bridge::{LuaList, ld_int, ld_bool, ld_str};
use crate::voxels::Direction;
use crate::read_or_bail;

type InvResult = Result<LuaList, anyhow::Error>;


pub struct Peripherals {
    pub screen: crate::screen::Screen<90, 35>,
    pub gpu:    (Color32, Color32),
    pub sleep:  f64,
    pub timer:  f64,
    
    pub ores:   crate::voxels::Chunk<16>,
}

impl Peripherals {
    fn require_pause(&mut self, dt: f64) {
        self.sleep = self.sleep.max(self.timer + dt);
    }
}

impl Default for Peripherals {
    fn default() -> Self {
        Peripherals {
            screen: Default::default(),
            gpu:    (Color32::WHITE, Color32::DARK_BLUE),
            sleep:  f64::NEG_INFINITY,
            timer:  f64::NAN,
            
            ores:   Default::default(),
        }
    }
}


fn lua_direction(p: &Peripherals, args: &LuaList) -> anyhow::Result<Direction> {
    read_or_bail!([side] = args);
    
    Ok(match ld_int(side) {
        Ok(0) => Direction::Down,
        Ok(1) => Direction::Up,
        Ok(2) => {
            let mut d = p.ores.robot_dir;
            d.turn_around();
            Direction::Ground(d)
        },
        Ok(3) => Direction::Ground(p.ores.robot_dir),
        Ok(e) => return Err(anyhow!("bad direction cardinal {e}")),
        Err(e) => return Err(e.context("direction cardinal")),
    })
}



pub type Reactions = HashMap<
    (String, String),
    fn(&mut Peripherals, &LuaList) -> InvResult
>;

pub fn build_reactions() -> Reactions {
    use piccolo::Constant;
    
    let mut reactions = HashMap::new();
    reactions.insert(
        ("$gpu".to_owned(), "gpu.set".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            read_or_bail!([cx, cy, cs, cd] = args);
            let mut pos = (ld_int(cx)?.saturating_sub(1),
                           ld_int(cy)?.saturating_sub(1));
            let d = ld_bool(cd)?;
            
            if let Ok(one_char) = ld_int(cs) {
                let c = char::try_from(u32::try_from(one_char)?)?;
                p.screen.put(c, p.gpu, &mut pos, d);
            } else {
                for c in ld_str(cs)?.chars() {
                    p.screen.put(c, p.gpu, &mut pos, d);
                }
            }
            Ok(vec![])
        } as _
    );
    
    reactions.insert(
        ("$gpu".to_owned(), "gpu.getResolution".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            read_or_bail!([] = args);
            
            let _screen: &crate::screen::Screen<90, 35> = &p.screen;
            Ok(vec![
                Constant::Integer(90),
                Constant::Integer(35),
            ])
        } as _
    );
    
    reactions.insert(
        ("$gpu".to_owned(), "gpu.setForeground".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            read_or_bail!([color] = args);
            let color = color.to_integer().ok_or_else(|| anyhow!("int conversion failed"))?;
            let r = u8::try_from(color / 65536)?;
            let g = u8::try_from((color % 65536) / 256)?;
            let b = u8::try_from(color % 256)?;
            p.gpu.0 = Color32::from_rgb(r, g, b);
            Ok(vec![])
        } as _
    );
    
    reactions.insert(
        ("$gpu".to_owned(), "gpu.setBackground".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            read_or_bail!([color] = args);
            let color = color.to_integer().ok_or_else(|| anyhow!("int conversion failed"))?;
            let r = u8::try_from(color / 65536)?;
            let g = u8::try_from((color % 65536) / 256)?;
            let b = u8::try_from(color % 256)?;
            p.gpu.1 = Color32::from_rgb(r, g, b);
            Ok(vec![])
        } as _
    );
    
    reactions.insert(
        ("$os".to_owned(), "os.sleep".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            read_or_bail!([time] = args);
            p.sleep = p.timer + time.to_number().ok_or_else(|| anyhow!("number conversion failed"))?;
            Ok(vec![])
        } as _
    );
    
    
    // OK durability():number
    // OK move(direction:number):boolean
    // OK turn(clockwise:boolean):boolean
    // OK name():string
    // 33 swing(side:number[, face:number=side[, sneaky:boolean=false]]):boolean[, string]
    // no use(side:number[, face:number=side[, sneaky:boolean=false[, duration:number=0]]]): boolean[, string]
    // no place(side:number[, face:number=side[, sneaky:boolean=false]]): boolean[, string]
    // no getLightColor():number
    // no setLightColor(value:number):number
    
    reactions.insert(
        ("$robot".to_owned(), "robot.durability".to_owned()),
        |_: &mut Peripherals, args: &LuaList| -> InvResult {
            read_or_bail!([] = args);
            Ok(vec![
                Constant::Number(f64::INFINITY)
            ])
        } as _
    );
    
    reactions.insert(
        ("$robot".to_owned(), "robot.move".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            let dir = lua_direction(p, args)?;
            let success = Constant::Boolean(p.ores.robot_move(Some(dir)));
            // p.require_pause(0.02);
            Ok(vec![ success ])
        } as _
    );
    
    reactions.insert(
        ("$robot".to_owned(), "robot.turn".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            read_or_bail!([clockwise] = args);
            if ld_bool(clockwise)? {
                p.ores.robot_dir.turn_right();
            } else {
                p.ores.robot_dir.turn_left();
            }
            // p.require_pause(0.03);
            Ok(vec![ Constant::Boolean(true) ])
        } as _
    );
    
    reactions.insert(
        ("$robot".to_owned(), "robot.hidden$turnaround".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            read_or_bail!([] = args);
            p.ores.robot_dir.turn_around();
            p.require_pause(0.03);
            Ok(vec![ Constant::Boolean(true) ])
        } as _
    );
    
    reactions.insert(
        ("$robot".to_owned(), "robot.name".to_owned()),
        |_: &mut Peripherals, args: &LuaList| -> InvResult {
            read_or_bail!([] = args);
            Ok(vec![ Constant::String(b"Pickitten".to_owned().into()) ])
        } as _
    );
    
    reactions.insert(
        ("$robot".to_owned(), "robot.swing".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            let dir = lua_direction(p, args)?;
            
            match p.ores.robot_swing(Some(dir)) {
                None    => Ok(vec![Constant::Boolean(false)]),
                Some(broke_block) => {
                    if broke_block.is_tough() {
                        p.require_pause(0.03);
                    }
                    Ok(vec![
                        Constant::Boolean(true),
                        Constant::String(broke_block.destroy_to_itemid()),
                    ])
                },
            }
        } as _
    );
    
    reactions.insert(
        ("$robot".to_owned(), "robot.hidden$compareAir".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            let dir = lua_direction(p, args)?;
            let faced = p.ores.robot_faces(Some(dir)).map(|p| &*p.0);
            
            let template = &crate::block_registry::Voxel::Air;
            
            Ok(vec![
                Constant::Boolean(faced == Some(template))
            ])
        } as _
    );
    
    reactions.insert(
        ("$robot".to_owned(), "robot.hidden$compareRocks".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            let dir = lua_direction(p, args)?;
            let faced = p.ores.robot_faces(Some(dir)).map(|p| &*p.0);
            
            let template = &crate::block_registry::Voxel::Rocks;
            
            Ok(vec![
                Constant::Boolean(faced == Some(template))
            ])
        } as _
    );
    
    reactions
}

