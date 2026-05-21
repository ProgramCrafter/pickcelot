use std::collections::HashMap;
use egui::*;


const PADDING: f32 = 16.0;
const FONT_SIZE: f32 = 17.0;

const LUA_CODE: &[u8] = b"
local function gpu_set(...)
    return piccolo_component_invoke('$gpu', 'gpu.set', ...)
end

local s = '0123456789abcdefghiklmnopqrstuvwxyz'
s = s .. '!treecollectionsmeanthebestofthebest$' .. s
local i = 0
while true do
    i = i + 1
    if i > 90 then i = 1 end
    
    gpu_set(i + 1, 1, string.sub(s, i, i), false)
    gpu_set(i, 1, ' ', false)
    piccolo_yield()
end
";


mod screen {
    use crate::*;
    
    type ScreenChar = (char, Color32, Color32);
    const BLANK: ScreenChar = ('i', Color32::WHITE, Color32::DARK_BLUE);
    
    pub struct Screen<const W: usize, const H: usize> {
        pub chars: Box<[[ScreenChar; W]; H]>,
    }
    
    impl<const W: usize, const H: usize> Default for Screen<W, H> {
        fn default() -> Self { Self::new() }
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
    }
    
    impl<'a, const W: usize, const H: usize> Widget for &'a Screen<W, H> {
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
}


type InvocationVariadic = Vec<piccolo::Constant<Box<[u8]>>>;
struct ComponentInvoke {
    target:   (String, String),
    variadic: InvocationVariadic,
}
impl ComponentInvoke {
    #[must_use]
    fn make<'gc, 'a>(
        ctx: piccolo::Context<'gc>,
        stack: &mut piccolo::Stack<'gc, 'a>,
    ) -> Result<(), piccolo::TypeError> {
        use piccolo::*;
        use std::string::String;
        type Manyval<'gc> = Variadic<Vec<Value<'gc>>>;
        
        let args: Result<(String,String,Manyval), _> = stack.consume(ctx);
        let (address, method, variadic_ref) = args?;
        let variadic = variadic_ref.0.into_iter().map(|val| {
            let c = val.to_constant().unwrap_or(Constant::Nil);
            c.map_string(|s| s.as_bytes().into())
        }).collect();
        
        let ud = UserData::new_static(&ctx, ComponentInvoke {
            target: (address, method),
            variadic,
        });
        stack.replace(ctx, ud);
        
        Ok(())
    }
}



struct Peripherals {
    screen: screen::Screen<90, 35>,
    gpu:    (Color32, Color32),
}

struct App {
    config: piccolo::Lua,
    engine: piccolo::StashedExecutor,
    
    peripheral: Peripherals,
    components: HashMap<(String, String),
        fn(&mut Peripherals, &InvocationVariadic) -> bool>,
}

impl Default for App {
    fn default() -> Self {
        // 1. Initialize Lua part with callbacks.
        // no: IO
        // ok: base, coroutine, math, string, table
        let mut config = piccolo::Lua::core();
        
        let engine = config.enter(|ctx| {
            use piccolo::*;
            
            let piccolo_yield = Callback::from_fn(&ctx, |_, mut exec, _| {
                exec.fuel().interrupt();
                Ok(CallbackReturn::Return)
            });
            _ = ctx.set_global("piccolo_yield", piccolo_yield).unwrap();
            
            let ci = Callback::from_fn(&ctx, |ctx, _, mut stack| {
                ComponentInvoke::make(ctx, &mut stack)?;
                Ok(CallbackReturn::Yield {to_thread: None, then: None})
            });
            _ = ctx.set_global("piccolo_component_invoke", ci).unwrap();
            
            let cl = Closure::load(ctx, None, LUA_CODE).unwrap();
            let ex = Executor::start(ctx, cl.into(), ());
            ctx.stash(ex)
        });
        
        // 2. Initialize peripherals.
        let peripheral = Peripherals {
            screen: Default::default(),
            gpu:    (Color32::WHITE, Color32::DARK_BLUE),
        };
        
        // 3. Callbacks for peripherals.
        
        let mut components = HashMap::new();
        components.insert(("$gpu".to_owned(), "gpu.set".to_owned()),
        |p: &mut Peripherals, args: &InvocationVariadic| -> bool {
            use piccolo::Constant::*;
            
            let (cx, cy, cs, cd) = match &args[..] {
                [cx, cy, cs, cd] => (cx, cy, cs, cd),
                _                => return false,
            };
            
            let x = match cx {
                Integer(n) => *n,
                Number(n)  => n.trunc() as i64,
                _          => return false,
            };
            let y = match cy {
                Integer(n) => *n,
                Number(n)  => n.trunc() as i64,
                _          => return false,
            };
            let s = match cs {
                String(s)  => s,
                _          => return false,
            };
            let d = match cd {
                Boolean(d) => *d,
                _          => return false,
            };
            
            let Ok(x) = usize::try_from(x - 1) else {return false};
            let Ok(y) = usize::try_from(y - 1) else {return false};
            let Ok(s) = str::from_utf8(&s) else {return false};
            
            let gpu_config = p.gpu;
            
            for (i, c) in s.chars().enumerate() {
                let (char_x, char_y) = match d {
                    false => (x.saturating_add(i), y),
                    true  => (x, y.saturating_add(i)),
                };
                if let Some(row) = p.screen.chars.get_mut(char_y) {
                    if let Some(col) = row.get_mut(char_x) {
                        *col = (c, gpu_config.0, gpu_config.1);
                    }
                }
            }
            
            true
        } as _);
        
        
        Self {
            config,
            engine,
            peripheral,
            components,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ectx: &Context, _frame: &mut eframe::Frame) {
        self.config.enter(|ctx| {
            use piccolo::*;
            
            let ex = ctx.fetch(&self.engine);
            if ex.step(ctx, &mut Fuel::with(16384)) {
                let request = ex
                    .take_result::<UserData>(ctx)
                    .map(|ok_mode| ok_mode.map(
                        |ud| ud.downcast_static::<ComponentInvoke>()
                    ));
                
                let iv: &ComponentInvoke = match request {
                    Err(bad_exec) => panic!("bad_exec={bad_exec}"),
                    Ok(Err(thread)) => panic!("thread_err={thread}"),
                    Ok(Ok(Err(_bad_ud))) => panic!("not ComponentInvoke"),
                    Ok(Ok(Ok(iv))) => iv,
                };
                
                if let Some(cb) = self.components.get(&iv.target)
                    && (cb)(&mut self.peripheral, &iv.variadic) {
                    ex.resume(ctx, ()).expect("yielded, cb complete");
                } else {
                    let s = String::from_slice(&ctx, "bad invocation");
                    let e = Error::from_value(s.into());
                    ex.resume_err(&ctx, e).expect("yielded, to resume");
                }
            }
            
            if ex.mode() != ExecutorMode::Stopped {
                ectx.request_repaint_after(
                    std::time::Duration::from_millis(90)
                );
            }
        });
        
        CentralPanel::default()
            .frame(Frame::new().inner_margin(PADDING))
            .show(ectx, |ui| {
                ui.add(&self.peripheral.screen);
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1032.0, 832.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Screen",
        options,
        Box::new(|_| Ok(Box::new(App::default()))),
    )
}
