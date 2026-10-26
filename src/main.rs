use egui::*;

mod block_registry;
mod block_colors;
mod sample_code;
mod peripherals;
mod voxels;
mod screen;
mod bridge;

use crate::sample_code::LUA_CODE;
use crate::bridge::ComponentInvoke;

const PADDING: f32 = 16.0;


struct App {
    config: piccolo::Lua,
    engine: piccolo::StashedExecutor,
    
    peripheral: peripherals::Peripherals,
    components: peripherals::Reactions,
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
        
        Self {
            config,
            engine,
            // 2. Initialize peripherals.
            peripheral: Default::default(),
            // 3. Callbacks for peripherals.
            components: peripherals::build_reactions(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, eui: &mut Ui, _frame: &mut eframe::Frame) {
        self.peripheral.timer = eui.ctx().input(|i| i.time);
        
        let mut invocations = 0;
        
        self.config.enter(|ctx| {
            use piccolo::*;
            
            let sleep_t = self.peripheral.sleep - self.peripheral.timer;
            if sleep_t > 0.0 {
                eui.ctx().request_repaint_after_secs(sleep_t as f32);
                return;
                
                // NOTE: why not in scope of `ui`?
                // We shouldn't skip rendering, and it is syntactically easier
                // to first enter gc context and then return if still asleep.
            }
            self.peripheral.sleep = f64::NEG_INFINITY;
            
            let ex = ctx.fetch(&self.engine);
            let mut fuel = Fuel::with(4096 * 30);
            while ex.step(ctx, &mut fuel) {
                let request = ex
                    .take_result::<UserData>(ctx)
                    .map(|ok_mode| ok_mode.map(
                        |ud| ud.downcast_static::<ComponentInvoke>()
                    ));
                
                let iv: &ComponentInvoke = match request {
                    Err(bad_exec) => panic!("bad_exec={bad_exec}"),
                    Ok(Err(thread)) => panic!("thread_err={thread:?}"),
                    Ok(Ok(Err(_bad_ud))) => panic!("not ComponentInvoke"),
                    Ok(Ok(Ok(iv))) => iv,
                };
                
                invocations += 1;
                
                if let Some(cb) = self.components.get(&iv.target) {
                    match (cb)(&mut self.peripheral, &iv.variadic) {
                        Ok(v) => ex.resume(ctx, ComponentInvoke::stash_vec(ctx, v)),
                        Err(e) => ex.resume_err(&ctx, e.into()),
                    }.expect("resuming should work because callback can return");
                    
                    if self.peripheral.sleep > self.peripheral.timer {
                        break;
                    }
                } else {
                    let s = String::from_slice(&ctx, "bad invocation");
                    let e = Error::from_value(s.into());
                    ex.resume_err(&ctx, e).expect("yielded, to resume");
                }
            }
            
            if ex.mode() != ExecutorMode::Stopped {
                eui.ctx().request_repaint_after_secs(0.04);
            }
        });
        
        Panel::right("ores")
            .frame(Frame::new().inner_margin(PADDING))
            .show(eui, |ui| {
                ui.add(&self.peripheral.ores);
            });
        
        CentralPanel::default()
            .frame(Frame::new().inner_margin(PADDING))
            .show(eui, |ui| {
                ui.add(&self.peripheral.screen);
                ui.add_space(PADDING);
                
                if invocations != 0 {
                    ui.label(format!("component.invoke(): {invocations}"));
                } else {
                    ui.label("component.invoke(): ___");
                }
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1632.0, 832.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Screen",
        options,
        Box::new(|_| Ok(Box::new(App::default()))),
    )
}
