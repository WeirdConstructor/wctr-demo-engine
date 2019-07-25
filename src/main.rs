mod turtle;
mod signals;
mod clcontext;

use clcontext::WLambdaCtx;
use turtle::{TurtleDrawing, ShapeRotation};
//use std::time::{Instant};

use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics;

/* TODO:

    X use turtle state
    X implement turtle color
    X implement turtle vector direction
    X implement turtle line drawing
    - check out filemanager project GUI for possible
      utilization as tracker.
    - implement gradient Op with 4 outputs
    - implement layered noise buffer using xorshift crate,
      which can be sampled by register accesses
        - implement textured rects and possibly display the noise buffer.


*/
/*

Design ideas:

Basic building blocks:

    - Timeline, depending on seconds?
    - Signal-Generator-Nodes, that get T as input and output a float
        - Constant
        - Interpolation/Tween
            - linear, smoothstep, sin/cos
        - random numbers, interpolated maybe?
        - start time
        - end time
        - initial value
    - Signal Mod Nodes
        - Mapper node of 1 value to something inside a range
    - Draw nodes:
        - Pixel
            - Input
                - HSV Color
                - X/Y
        - Rectangle
            - Input
                - HSV Color
                - X/Y
                - W/H | X2/Y2
        - Filled Rectangle
            - Input
                - HSV Color
                - X/Y
                - W/H | X2/Y2
        - Sprite
            - X/Y
            - W/H
            - Blend Mode
            - Color Modifier
    - Controls
        - show time
        - single time step
        - restart
        - set loop start/end
        - play/stop
        - reload graph by discarding the nodes and reevaluating the script
            - do that automatically when the mtime of the script
              changes.
    - Tracker Input
*/

struct Painter<'a> {
    ctx: &'a mut Context,
}

impl<'a> TurtleDrawing for Painter<'a> {
    fn draw_line(&mut self, color: [f32; 4], _rot: ShapeRotation, from: [f32; 2], to: [f32; 2], thickness: f32) {
//        println!("COLO {:?} => {:?}", color, graphics::Color::from(color));
        let line = graphics::Mesh::new_line(self.ctx, &[[0.0, 0.0], [to[0] - from[0], to[1] - from[1]]], 2.0, graphics::Color::from(color)).unwrap();
        graphics::draw(self.ctx, &line, (from, 0.0, [0.0, 0.0], graphics::WHITE));
//        let o = Ellipse::new(color);
//        o.draw(
//            ellipse::circle(from[0] as f64, from[1] as f64, thickness as f64),
//            &self.ctx.draw_state, self.ctx.transform, self.g);
//        o.draw(
//            ellipse::circle(to[0] as f64, to[1] as f64, thickness as f64),
//            &self.ctx.draw_state, self.ctx.transform, self.g);
//        line_from_to(
//            color, thickness as f64,
//            [from[0] as f64, from[1] as f64],
//            [to[0] as f64, to[1] as f64],
//            self.ctx.transform, self.g);
    }

    fn draw_rect_fill(&mut self, color: [f32; 4], rot: ShapeRotation, pos: [f32; 2], size: [f32; 2]) {
//        let rot = match rot {
//            ShapeRotation::Center(a) => a,
//            _ => 0.0,
//        };
//        rectangle(
//            color,
//            [(pos[0] - size[0] / 2.0) as f64,
//             (pos[1] - size[1] / 2.0) as f64,
//             size[0] as f64,
//             size[1] as f64],
//            self.ctx.transform.rot_rad(rot as f64),
//            self.g);
    }
}

struct WCtrDemEngine {
    wlctx: WLambdaCtx,
}

impl WCtrDemEngine {
    pub fn new(ctx: &mut Context) -> WCtrDemEngine {
        let mut wlctx = WLambdaCtx::new();
        wlctx.init();
        wlctx.load_script("in.wl");
        WCtrDemEngine { wlctx }
    }
}

impl EventHandler for WCtrDemEngine {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let param = graphics::DrawParam::from(([0.0, 0.0], 0.0, [10.0, 100.0], graphics::BLACK));
        graphics::push_transform(ctx, Some(param.to_matrix()));
        graphics::apply_transformations(ctx)?;

        let now_time = ggez::timer::time_since_start(ctx).as_millis();
        let scale_size = 200.0;
        {
            let mut p = Painter { ctx };
            self.wlctx.one_step(now_time as i64, scale_size, &mut p);
        }

        graphics::present(ctx)
    }
}

fn main() {
    // Make a Context and an EventLoop.
    let (mut ctx, mut event_loop) =
       ContextBuilder::new("wctr_dem_engine", "Weird Constructor")
            .window_setup(ggez::conf::WindowSetup {
                title: "wctr_dem_engine".to_owned(),
                samples: ggez::conf::NumSamples::Four,
                vsync: true,
                icon: "".to_owned(),
                srgb: true,
            })
           .build()
           .unwrap();

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object
    // so it can load resources like images during setup.
    let mut engine = WCtrDemEngine::new(&mut ctx);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut engine) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}

//pub fn main() -> Result<(), String> {
//    let opengl = OpenGL::V3_2;
//    let mut window: PistonWindow =
//        WindowSettings::new("Hello Piston!", [640, 480])
//        .graphics_api(opengl)
//        .resizable(true)
//        .vsync(true)
//        .exit_on_esc(true).build().unwrap();
//
//    let mut cnt = 0;
//    let mut avg = 0;
//
//    let mut wlctx = WLambdaCtx::new();
//    wlctx.init();
//    wlctx.load_script("in.wl");
//
//    let start_time = Instant::now();
//    while let Some(event) = window.next() {
//        let ws = window.draw_size();
//
//        window.draw_2d(&event, |mut context, graphics, _device| {
//            clear([0.1; 4], graphics);
//
//            let scale_size = 200.0 as f32;
//
//            context.transform =
//                context.transform.trans(
//                    ws.width / 2.0 as f64,
//                    ws.height / 2.0 as f64);
//
//            let mut p = Painter {
//                ctx: &context,
//                g: graphics,
//            };
//
//            let b = Instant::now();
//
//            let now_time = start_time.elapsed().as_millis();
//
//            wlctx.one_step(now_time as i64, scale_size, &mut p);
//
//            avg += b.elapsed().as_millis();
//            cnt += 1;
//            if cnt > 100 {
//                println!("exec took {}", avg / cnt);
//                cnt = 0;
//                avg = 0;
//            }
//
//        });
//    }
//
//    Ok(())
//}
