use crate::signals::*;
use crate::turtle::*;

use wlambda;
use wlambda::vval::VVal;
use wlambda::prelude::create_wlamba_prelude;
use wlambda::vval::{Env};

use std::rc::Rc;
use std::cell::RefCell;

use crate::turtle::TurtleDrawing;

pub struct ClContext {
    sim:             Simulator,
    cur_turtle_cmds: Vec<Turtle>,
    turtle_stack:    Vec<Vec<Turtle>>,
}

impl ClContext {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(ClContext {
            sim: Simulator {
                ops:  Vec::new(),
                regs: Vec::new(),
            },
            cur_turtle_cmds: Vec::new(),
            turtle_stack:    Vec::new(),
        }))
    }

    fn new_op(&mut self, idx: usize, t: &str) -> Option<usize> {
        let sim = &mut self.sim;
        let mut o : Box<dyn DemOp> = match t {
            "sin" => { Box::new(DoSin::new()) },
            _     => { return None; },
        };

        let new_start_reg = sim.regs.len();
        sim.regs.resize(sim.regs.len() + o.output_count(), 0.0);
        o.init_regs(new_start_reg, &mut sim.regs[..]);
        let out_reg = o.get_output_reg("out");

        sim.ops.insert(idx, o);

        out_reg
    }

    fn set_reg(&mut self, idx: usize, v: f32) -> bool {
        if self.sim.regs.len() > idx {
            self.sim.regs[idx] = v;
            true
        } else {
            false
        }
    }

    fn get_reg(&self, idx: usize) -> f32 {
        if self.sim.regs.len() > idx {
            self.sim.regs[idx]
        } else {
            0.0
        }
    }

    fn exec(&mut self, t: f32) {
        for r in self.sim.ops.iter_mut() {
            r.as_mut().exec(t, &mut self.sim.regs[..]);
        }
    }

    fn pack_turtle(&mut self) {
        let t =
            Turtle::Commands(
                std::mem::replace(&mut self.cur_turtle_cmds, Vec::new()));
        self.cur_turtle_cmds.push(t);
    }

    fn add_turtle(&mut self, t: Turtle) {
        println!("ADD TURTLE: {:?}", t);
        self.cur_turtle_cmds.push(t);
    }

    fn push_turtle(&mut self) {
        self.turtle_stack.push(
            std::mem::replace(&mut self.cur_turtle_cmds, Vec::new()));
    }

    fn pop_turtle(&mut self) -> Turtle {
        let prev_t = self.turtle_stack.pop().unwrap();
        Turtle::Commands(
            std::mem::replace(&mut self.cur_turtle_cmds, prev_t))
    }
}

macro_rules! getOpIn {
    ($arg: ident, $o: ident) => {
        let $o = if let Some(o) = OpIn::vv2opin($arg.clone()) {
            o
        } else {
            return Ok(VVal::err_msg(&format!("Bad register '{}'", $arg.s())));
        };
    }
}

macro_rules! getColorIn {
    ($arg: ident, $o: ident) => {
        let v = $arg.clone();
        if !v.is_vec() {
            return Ok(VVal::err_msg(
                &format!("Bad color argument '{}'", $arg.s())));
        }

        let $o = ColorIn {
            h: if let Some(o) = OpIn::vv2opin(v.at(0).unwrap_or(VVal::Nul)) {
                o
            } else {
                return Ok(VVal::err_msg(
                    &format!("Bad register '{}'",
                             v.at(0).unwrap_or(VVal::Nul).s())));
            },
            s: if let Some(o) = OpIn::vv2opin(v.at(1).unwrap_or(VVal::Nul)) {
                o
            } else {
                return Ok(VVal::err_msg(
                    &format!("Bad register '{}'",
                             v.at(1).unwrap_or(VVal::Nul).s())));
            },
            v: if let Some(o) = OpIn::vv2opin(v.at(2).unwrap_or(VVal::Nul)) {
                o
            } else {
                return Ok(VVal::err_msg(
                    &format!("Bad register '{}'",
                             v.at(2).unwrap_or(VVal::Nul).s())));
            },
            a: if let Some(o) = OpIn::vv2opin(v.at(3).unwrap_or(VVal::Nul)) {
                o
            } else {
                return Ok(VVal::err_msg(
                    &format!("Bad register '{}'",
                             v.at(3).unwrap_or(VVal::Nul).s())));
            },
        };
    }
}

pub struct WLambdaCtx {
    clctx:      Rc<RefCell<ClContext>>,
    evalctx:    Option<wlambda::compiler::EvalContext>,
    draw_cb:    VVal,
}

impl WLambdaCtx {
    pub fn new() -> Self {
        WLambdaCtx {
            clctx: ClContext::new(),
            evalctx: None,
            draw_cb: VVal::Nul,
        }
    }

    pub fn init(&mut self) {
        let genv = create_wlamba_prelude();
        genv.borrow_mut().add_func(
            "t", |env: &mut Env, _argc: usize| {
                let node_type = env.arg(0).s_raw();
                let a1 = env.arg(1).clone();
                let a2 = env.arg(2).clone();
                let a3 = env.arg(3).clone();

                match &node_type[..] {
                    "cmds" => {
                        env.with_user_do(|clx: &mut ClContext| clx.pack_turtle());
                    },
                    // TODO!
    //                    "area" => {
    //                        getOpIn!(a1, aw);
    //                        getOpIn!(a2, ah);
    //                        clx.pack_turtle();
    //                        let t = Box::new(clx.cur_turtle_cmds.pop().unwrap());
    //                        clx.add_turtle(Turtle::Area((aw, ah), t));
    //                    },
    //                  "
                    "with_state" => {
                        env.with_user_do(|clx: &mut ClContext|
                            clx.push_turtle());
                        match a1.call_no_args(env) {
                            Ok(_v) => {
                                env.with_user_do(|clx: &mut ClContext| {
                                    let t = Turtle::WithState(Box::new(clx.pop_turtle()));
                                    clx.add_turtle(t);
                                });
                            },
                            Err(e) => return Err(e),
                        }
                    },
                    "look_dir" => {
                        getOpIn!(a1, x);
                        getOpIn!(a2, y);

                        env.with_user_do(|clx: &mut ClContext|
                            clx.add_turtle(Turtle::LookDir(x, y)));
                    },
                    "line" => {
                        getOpIn!(a1, n);
                        getOpIn!(a2, t);
                        getColorIn!(a3, clr);

                        env.with_user_do(|clx: &mut ClContext|
                            clx.add_turtle(Turtle::Line(n, t, clr)));
                    },
                    "rect" => {
                        getOpIn!(a1, w);
                        getOpIn!(a2, h);
                        getColorIn!(a3, clr);

                        env.with_user_do(|clx: &mut ClContext|
                            clx.add_turtle(Turtle::Rect(w, h, clr)));
                    },
                    _ => {
                        return Ok(VVal::err_msg(
                            &format!("Bad turtle type '{}'", node_type)))
                    }
                }

                Ok(VVal::Bol(true))
            }, Some(1), None);

        genv.borrow_mut().add_func(
            "reg", |env: &mut Env, argc: usize| {
                let reg = env.arg(0).i() as usize;
                let val = env.arg(1).f() as f32;

                if argc > 1 {
                env.with_user_do(|clx: &mut ClContext| {
                        clx.set_reg(reg, val);
                    });
                    Ok(VVal::Bol(true))
                } else {
                    Ok(VVal::Flt(env.with_user_do(|clx: &mut ClContext| {
                        clx.get_reg(reg)
                    }) as f64))
                }
            }, Some(1), Some(2));

        genv.borrow_mut().add_func(
            "new", |env: &mut Env, _argc: usize| {
                let idx = env.arg(0).i() as usize;
                let t   = env.arg(1).s_raw();

                env.with_user_do(|clx: &mut ClContext| {
                    let o = clx.new_op(idx, &t);
                    if let Some(i) = o {
                        Ok(VVal::Int(i as i64))
                    } else {
                        Ok(VVal::err_msg(&format!("Bad op type '{}'", t)))
                    }
                })
            }, Some(2), Some(2));

        self.evalctx =
            Some(
                wlambda::compiler::EvalContext::new_with_user(
                    genv, self.clctx.clone()));

    }

    pub fn load_script(&mut self, filename: &str) {
        self.evalctx.as_mut().unwrap().eval_file(
            &std::env::args().nth(1).unwrap_or(filename.to_string())).unwrap();

        let draw_cb = self.evalctx.as_mut().unwrap().get_global_var("draw");
        if draw_cb.is_none() {
            panic!("script did not setup a global draw() function!");
        }
        let draw_cb = draw_cb.unwrap();
        if !draw_cb.is_fun() {
            panic!("script did not setup a global draw() function!");
        }

        self.draw_cb = draw_cb;
    }

    pub fn one_step<T>(&mut self, t: i64, scale_size: f32, p: &mut T) where T: TurtleDrawing {
        self.evalctx.as_mut().unwrap().call(
            &self.draw_cb,
            &vec![VVal::Int(t)]).unwrap();
        self.clctx.borrow_mut().exec(t as f32);

        let t = self.clctx.borrow_mut().cur_turtle_cmds[0].clone();

        let mut ts = TurtleState::new(scale_size, scale_size);
        t.exec(&mut ts, &self.clctx.borrow().sim.regs, p);
    }
}