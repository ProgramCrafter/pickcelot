#[repr(u8)]
#[non_exhaustive]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Voxel { Air = 0, Stone, Ore }

impl Voxel {
    pub fn is_tough(&self) -> bool {
        self != &Voxel::Air
    }
    
    pub fn destroy_to_itemid(self) -> Box<[u8]> {
        match self {
            Self::Air => b"pickcelot:air".to_owned().into(),
            Self::Stone => b"pickcelot:stone".to_owned().into(),
            Self::Ore => b"pickcelot:ore".to_owned().into(),
        }
    }
    
    pub fn summon_grid<const S: usize>(
        blank_pos: (usize, usize, usize),
    ) -> Box<[[[Self; S]; S]; S]> {
        use std::array::from_fn;
        
        // let (x, y, z) = blank_pos;
        // let mut a = Box::new([[[Self::Stone; S]; S]; S]);
        // if x < S && y < S && z < S { a[z][y][x] = Self::Air; }
        
        Box::new(
            from_fn(|z|
                from_fn(|y|
                    from_fn(|x|
                        if (x,y,z) == blank_pos {Self::Air} else {Self::Stone}
                    )
                )
            )
        )
    }
}

