use vecmath;
use crate::signals::OpIn;
use crate::signals::ColorIn;

#[derive(Debug, PartialEq, Clone)]
pub enum Turtle {
    Commands(Vec<Turtle>),
    LookDir(OpIn, OpIn),
    WithState(Box<Turtle>),
    Rect(OpIn, OpIn, ColorIn),
    Line(OpIn, OpIn, ColorIn),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ShapeRotation {
    LeftBottom(f32),
    TopRight(f32),
    Center(f32),
}

#[derive(Debug, PartialEq, Clone)]
pub struct TurtleState {
    w:          f32,
    h:          f32,
    pos:        [f32; 2],
    dir:        [f32; 2],
}

impl TurtleState {
    pub fn new(w: f32, h: f32) -> Self {
        TurtleState {
            w,
            h,
            pos: [0.0, 0.0],
            dir: [0.0, 1.0],
        }
    }
}

pub trait TurtleDrawing {
    fn draw_line(&mut self, color: [f32; 4], rot: ShapeRotation, from: [f32; 2], to: [f32; 2], thickness: f32);
    fn draw_rect_fill(&mut self, color: [f32; 4], rot: ShapeRotation, pos: [f32; 2], size: [f32; 2]);
}

impl Turtle {
    pub fn exec<T>(&self,
               ts: &mut TurtleState,
               regs: &[f32],
               ctx: &mut T)
        where T: TurtleDrawing {
        match self {
            Turtle::Commands(v) => {
                for c in v.iter() {
                    c.exec(ts, regs, ctx);
                }
            },
            Turtle::WithState(cmds) => {
                let mut sub_ts = ts.clone();
                cmds.exec(&mut sub_ts, regs, ctx);
            },
            Turtle::LookDir(x, y) => {
                let x = x.calc(regs);
                let y = y.calc(regs);
                ts.dir = [x as f32, y as f32];
                ts.dir = vecmath::vec2_normalized(ts.dir);
            },
            Turtle::Line(n, thick, color) => {
                let n     = n.calc(regs);
                let t     = thick.calc(regs);
                let color = color.calc(regs);
                let mut new_pos = vecmath::vec2_scale(ts.dir, n as f32);
                new_pos[0] = ts.pos[0] + new_pos[0] * ts.w;
                new_pos[1] = ts.pos[1] + new_pos[1] * ts.h;
                ctx.draw_line(
                    color,
                    ShapeRotation::LeftBottom(0.0),
                    ts.pos,
                    new_pos,
                    t.into());
                ts.pos = new_pos;
            },
            Turtle::Rect(rw, rh, clr) => {
                let w = rw.calc(regs) * ts.w;
                let h = rh.calc(regs) * ts.h;
                let c = clr.calc(regs);

//                let angle = vecmath::vec2_dot([0.0, 1.0], ts.dir).acos();
                let angle =
                    2.0*std::f32::consts::PI
                    - ((1.0 as f32).atan2(0.0)
                       - ts.dir[1].atan2(ts.dir[0]));
                println!("RO {} => {:?}", angle, ts.dir);

                ctx.draw_rect_fill(
                    c,
                    ShapeRotation::Center(angle),
                    [ts.pos[0], ts.pos[0]],
                    [w, h]);
            },
        }
    }
}
