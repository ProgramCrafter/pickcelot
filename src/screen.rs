use crate::*;

type ScreenChar = (char, Color32, Color32);
const BLANK: ScreenChar = ('i', Color32::WHITE, Color32::DARK_BLUE);
const FONT_SIZE: f32 = 17.0;

pub struct Screen<const W: usize, const H: usize> {
    pub chars: Box<[[ScreenChar; W]; H]>,
}

impl<const W: usize, const H: usize> Screen<W, H> {
    fn new() -> Self {
        Self {
            chars: Box::new([[BLANK; W]; H]),
        }
    }
    fn draw_callback(&self, mut painter: impl FnMut(f32, f32, ScreenChar)) {
        for y in 0..H {
            for (x, schar) in self.chars[y].iter().enumerate().rev() {
                painter(x as f32, y as f32, *schar);
            }
        }
    }
    
    pub fn put(&mut self, c: char, colors: (Color32, Color32),
               location: &mut (i64, i64), carriage_vertical: bool) {
        let (ix, iy) = *location;
        if let Ok(y) = usize::try_from(iy) && y < H
        && let Ok(x) = usize::try_from(ix) && x < W {
            self.chars[y][x] = (c, colors.0, colors.1);
        }
        
        if carriage_vertical {
            location.1 = iy.saturating_add(1);
        } else {
            let char_width = if c.is_control() { 0 } else { 1 };
            location.0 = ix.saturating_add(char_width);
        }
    }
}

impl<const W: usize, const H: usize> Default for Screen<W, H> {
    fn default() -> Self { Self::new() }
}

impl<const W: usize, const H: usize> Widget for &Screen<W, H> {
    fn ui(self, ui: &mut Ui) -> Response {
        // 1. Space requirements.
        let font_id = FontId::monospace(FONT_SIZE);
        let (cell_w, cell_h) = ui.ctx().fonts_mut(|f| {
            (f.glyph_width(&font_id, '~'), f.row_height(&font_id))
        });
        assert_ne!(cell_w, 0.0, "font missing characters like '~'");
        let grid = vec2(cell_w * (W as f32), cell_h * (H as f32));
        let cell = vec2(cell_w, cell_h);
        let sense = Sense::empty();
        
        // 2. Space.
        let (response, painter) = ui.allocate_painter(grid, sense);
        let rect = response.rect;
        
        // 3. Drawing.
        self.draw_callback(|x, y, (ch, fg, bg)| {
            let pos = rect.min + vec2(x * cell_w, y * cell_h);
            let cell_rect = Rect::from_min_size(pos, cell);

            painter.rect_filled(cell_rect, 0.0, bg);
            painter.text(pos, Align2::LEFT_TOP, ch, font_id.clone(), fg);
        });
        
        response
    }
}

