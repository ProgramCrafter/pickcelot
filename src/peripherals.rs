use std::collections::HashMap;
use anyhow::anyhow;
use egui::Color32;

use crate::bridge::*;//InvResult, LuaList};
type InvResult = Result<LuaList, anyhow::Error>;


pub struct Peripherals {
    pub screen: crate::screen::Screen<90, 35>,
    pub gpu:    (Color32, Color32),
    pub sleep:  f64,
    pub timer:  f64,
    
    pub ores:   crate::voxels::Chunk<16>,
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



pub type Reactions = HashMap<
    (String, String),
    fn(&mut Peripherals, &LuaList) -> InvResult
>;

pub fn build_reactions() -> Reactions {
    let mut reactions = HashMap::new();
    reactions.insert(
        ("$gpu".to_owned(), "gpu.set".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            let [cx, cy, cs, cd] = args.as_array().ok_or_else(asize)?;
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
            let [] = args.as_array().ok_or_else(|| anyhow!("wrong number of arguments passed: 0 expected"))?;
            
            let _screen: &crate::screen::Screen<90, 35> = &p.screen;
            Ok(vec![
                piccolo::Constant::Integer(90),
                piccolo::Constant::Integer(35),
            ])
        } as _
    );
    
    reactions.insert(
        ("$gpu".to_owned(), "gpu.setForeground".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            let [color] = args.as_array().ok_or_else(|| anyhow!("wrong number of arguments passed: 1 expected"))?;
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
            let [color] = args.as_array().ok_or_else(|| anyhow!("wrong number of arguments passed: 1 expected"))?;
            let color = color.to_integer().ok_or_else(|| anyhow!("int conversion failed"))?;
            let r = u8::try_from(color / 65536)?;
            let g = u8::try_from((color % 65536) / 256)?;
            let b = u8::try_from(color % 256)?;
            p.gpu.1 = Color32::from_rgb(r, g, b);
            Ok(vec![])
        } as _
    );
    
    reactions.insert(
        ("$robot".to_owned(), "robot.turnRight".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            let [] = args.as_array().ok_or_else(|| anyhow!("wrong number of arguments passed: 0 expected"))?;
            p.sleep = p.sleep.max(p.timer + 0.15);
            p.ores.robot_dir.turn_right();
            Ok(vec![])
        } as _
    );
    
    reactions.insert(
        ("$robot".to_owned(), "robot.swing".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            let [] = args.as_array().ok_or_else(|| anyhow!("wrong number of arguments passed: 0 expected"))?;
            
            match p.ores.robot_swing(None) {
                None    => Ok(vec![piccolo::Constant::Boolean(false)]),
                Some(broke_block) => {
                    if broke_block.is_tough() {
                        p.sleep = p.sleep.max(p.timer + 0.25);
                    }
                    Ok(vec![
                        piccolo::Constant::Boolean(true),
                        piccolo::Constant::String(broke_block.destroy_to_itemid()),
                    ])
                },
            }
        } as _
    );
    
    reactions.insert(
        ("$robot".to_owned(), "robot.forward".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            let [] = args.as_array().ok_or_else(|| anyhow!("wrong number of arguments passed: 0 expected"))?;
            
            p.sleep = p.sleep.max(p.timer + 0.02);
            
            Ok(vec![
                piccolo::Constant::Boolean(p.ores.robot_move(None)),
            ])
        } as _
    );
    
    reactions.insert(
        ("$os".to_owned(), "os.sleep".to_owned()),
        |p: &mut Peripherals, args: &LuaList| -> InvResult {
            let [time] = args.as_array().ok_or_else(|| anyhow!("wrong number of arguments passed: 1 expected"))?;
            p.sleep = p.timer + time.to_number().ok_or_else(|| anyhow!("number conversion failed"))?;
            Ok(vec![])
        } as _
    );
    
    reactions
}

