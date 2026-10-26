#[repr(u8)]
#[non_exhaustive]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Voxel { Air = 0, Rocks, Ore }

impl Voxel {
    pub fn is_tough(&self) -> bool {
        self != &Voxel::Air
    }
    
    pub fn destroy_to_itemid(self) -> Box<[u8]> {
        match self {
            Self::Air => b"pickcelot:air".to_owned().into(),
            Self::Rocks => b"pickcelot:rocks".to_owned().into(),
            Self::Ore => b"pickcelot:ore".to_owned().into(),
        }
    }
    
    pub fn summon_grid<const S: usize>(
        blank_pos: (usize, usize, usize),
    ) -> Box<[[[Self; S]; S]; S]> {
        use std::array::from_fn;
        
        let oregen: [[core::ops::Range<usize>; S]; S] = from_fn(|z|
            from_fn(|x| {
                let mut h = 0xcbf29ce484222325u64 as usize;
                let m = 0x00000100000001b3u64 as usize;
                h ^= x;
                h = h.wrapping_mul(m);
                h ^= z;
                h = h.wrapping_mul(m);
                let y0 = h % S;
                let y2 = h / S % S;
                
                h ^= y0;
                h = h.wrapping_mul(m);
                let y1 = h % S;
                let y3 = h / S % S;
                
                let (y_lo1, y_hi1) = (y0.min(y1), y0.max(y1) + 1);
                let (y_lo2, y_hi2) = (y2.min(y3), y2.max(y3) + 1);
                let (y_lo, y_hi) = (y_lo1.max(y_lo2), y_hi1.min(y_hi2));
                y_lo..y_hi.max(y_lo)
            })
        );
        
        Box::new(
            from_fn(|z|
                from_fn(|y|
                    from_fn(|x|
                        if (x,y,z) == blank_pos {
                            Self::Air
                        } else if oregen[z][x].contains(&y) {
                            Self::Ore
                        } else {
                            Self::Rocks
                        }
                    )
                )
            )
        )
    }
}

