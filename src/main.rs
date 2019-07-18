use sdl2::pixels::Color;
use sdl2::event::Event;
//use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
//use sdl2::rect::Point;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Instant};
use wlambda;
use wlambda::vval::VVal;

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

//struct DrawState<'a, 'b> {
//    canvas: sdl2::render::Canvas<sdl2::video::Window>,
//    font: Rc<RefCell<sdl2::ttf::Font<'a, 'b>>>,
//}
//
//impl<'a, 'b> DrawState<'a, 'b> {
//    fn clear(&mut self) {
//        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
//        self.canvas.clear();
//    }
//
//    fn done(&mut self) {
//        self.canvas.present();
//    }
//
//    fn calc_column_text_widths(&mut self, table: &mut Table) {
//        for col in table.columns.iter_mut() {
//            if let ColumnSizing::TextWidth(txt) = &col.size {
//                if col.calc_size.is_none() {
//                    let tsize = self.font.borrow().size_of(&txt);
//                    col.calc_size = Some(tsize.unwrap_or((0, 0)).0 as i32);
//                }
//            } else {
//                col.calc_size = Some(0);
//            }
//        }
//    }
//
//    fn calc_column_width(&mut self, table: &Table, table_width: i32, skip_cols: u32) -> std::vec::Vec<i32> {
//        if skip_cols >= table.columns.len() as u32 {
//            return Vec::new();
//        }
//
//        let cols : std::vec::Vec<&Column> = table.columns.iter().rev().skip(skip_cols as usize).rev().collect();
//
//        let fixed_width : i32 =
//            cols.iter().map(|c| c.calc_size.unwrap() + table.col_gap as i32).sum();
//
//        let expand_rest_width = table_width - fixed_width;
//
//        if expand_rest_width < MIN_EXPAND_WIDTH {
//            return self.calc_column_width(table, table_width, skip_cols + 1);
//        }
//
//        let fract_sum : u32 = cols.iter().map(|c|
//            match c.size {
//                ColumnSizing::ExpandFract(f) => f as u32,
//                _ => 0u32,
//            }).sum();
//
//        cols.iter().map(|column|
//            match column.size {
//                ColumnSizing::TextWidth(_)   => column.calc_size.unwrap() + table.col_gap as i32,
//                ColumnSizing::ExpandFract(f) => ((expand_rest_width * f) / fract_sum as i32) + table.col_gap as i32,
//            }).collect()
//    }
//
//    fn draw_table_row(&mut self, row: &StyleString,
//                      col_idx: i32,
//                      row_idx: usize,
//                      has_focus: bool,
//                      fm_page: &Rc<dyn FmPage>,
//                      x: i32,
//                      y: i32,
//                      width: i32,
//                      col_gap: i32,
//                      row_height: i32) {
//
//        let mut fg_color = match row.style {
//            Style::Dir     => DIR_FG_COLOR,
//            Style::Special => LNK_FG_COLOR,
//            _              => NORM_FG_COLOR,
//        };
//
//        let mut bg_color = if row_idx % 2 == 0 {
//            if col_idx % 2 == 0 { NORM_BG_COLOR } else { NORM_BG2_COLOR }
//        } else {
//            if col_idx % 2 == 0 { NORM_BG2_COLOR } else { NORM_BG3_COLOR }
//        };
//
//        if has_focus && fm_page.is_cursor_idx(row_idx) {
//            bg_color = CURS_BG_COLOR;
//            fg_color = CURS_FG_COLOR;
//
//        } else if fm_page.is_selected(row_idx) {
//            bg_color = SLCT_BG_COLOR;
//            fg_color = SLCT_FG_COLOR;
//
//        } else if fm_page.is_highlighted(row_idx) {
//            bg_color = HIGH_FG_COLOR;
//            fg_color = HIGH_FG_COLOR;
//        }
//
//        self.canvas.set_draw_color(bg_color);
//        self.canvas.fill_rect(Rect::new(x, y, width as u32, row_height as u32));
//        draw_bg_text(
//            &mut self.canvas,
//            &mut self.font.borrow_mut(),
//            fg_color, bg_color,
//            x, y, width - col_gap, row_height,
//            &row.text);
//    }
//
//    fn draw_table(
//        &mut self,
//        pg: &mut Page,
//        x_offs: i32,
//        y_offs: i32,
//        table_width: i32,
//        table_height: i32,
//        has_focus: bool) -> RenderFeedback {
//
//        let table = pg.cache.as_mut().unwrap();
//
//        self.calc_column_text_widths(table);
//        let cols = self.calc_column_width(table, table_width, 0);
//
//        let row_height = self.font.borrow().height() + table.row_gap as i32;
//
//        draw_bg_text(
//            &mut self.canvas, &mut self.font.borrow_mut(),
//            NORM_FG_COLOR, NORM_BG_COLOR,
//            x_offs, y_offs, table_width, row_height,
//            &table.title);
//
//        let y_offs = y_offs + row_height;
//
//        let mut x = x_offs;
//        for width_and_col in cols.iter().enumerate().zip(table.columns.iter()) {
//            let col_idx = (width_and_col.0).0;
//            let width   = (width_and_col.0).1;
//            let column  = width_and_col.1;
//            //d// println!("COL {}, w: {}, h: {}", col_idx, width, column.head);
//
//            draw_bg_text(
//                &mut self.canvas, &mut self.font.borrow_mut(),
//                NORM_FG_COLOR, NORM_BG_COLOR,
//                x, y_offs, *width - table.col_gap as i32, row_height,
//                &column.head);
//
//            self.canvas.set_draw_color(NORM_FG_COLOR);
//            self.canvas.draw_line(
//                Point::new(x,         y_offs + (row_height - table.row_gap as i32)),
//                Point::new(x + width, y_offs + (row_height - table.row_gap as i32)));
//
//            let mut y = y_offs + row_height;
//
//            for (row_idx, row) in column.rows.iter()
//                                    .enumerate()
//                                    .skip(pg.fm_page.get_scroll_offs()) {
//
//                if (y - y_offs) + row_height > table_height {
//                    break;
//                }
//
//                self.draw_table_row(
//                    row, col_idx as i32, row_idx, has_focus,
//                    &pg.fm_page,
//                    x, y,
//                    *width, table.col_gap as i32, row_height);
//
//                y += row_height;
//            }
//
//            x += width;
//            //d// println!("X= {}", x);
//        }
//
//        let line_count = ((table_height - row_height) / row_height) as i32;
//        RenderFeedback {
//            // substract 1 row_height for title bar
//            recent_line_count: line_count as usize,
//            row_offset: pg.fm_page.get_scroll_offs(),
//            start_rows: (x_offs,
//                         y_offs + row_height),
//            row_height,
//            end_rows:   (x_offs + table_width,
//                         y_offs + row_height + line_count as i32 * row_height),
//        }
//    }
//}

//fn draw_text(font: &mut sdl2::ttf::Font, color: Color, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, x: i32, y: i32, max_w: i32, txt: &str) {
//    let txt_crt = canvas.texture_creator();
//
//    let sf = font.render(txt).blended(color).map_err(|e| e.to_string()).unwrap();
//    let txt = txt_crt.create_texture_from_surface(&sf).map_err(|e| e.to_string()).unwrap();
//    let tq = txt.query();
//
//    let w : i32 = if max_w < (tq.width as i32) { max_w } else { tq.width as i32 };
//
////    txt.set_color_mod(255, 0, 0);
//    canvas.copy(
//        &txt,
//        Some(Rect::new(0, 0, w as u32, tq.height)),
//        Some(Rect::new(x, y, w as u32, tq.height))
//    ).map_err(|e| e.to_string()).unwrap();
//}
//
//fn draw_bg_text(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
//                font: &mut sdl2::ttf::Font,
//                color: Color,
//                bg_color: Color,
//                x: i32,
//                y: i32,
//                max_w: i32,
//                h: i32,
//                txt: &str) {
//
//    canvas.set_draw_color(bg_color);
//    canvas.fill_rect(Rect::new(x, y, max_w as u32, h as u32));
//    draw_text(font, color, canvas, x, y, max_w, txt);
//}

trait DemOp {
    fn alloc_regs(&self) -> usize;
    fn init_regs(&mut self, start_reg: usize, regs: &mut [f32]);
    fn get_reg(&mut self, name: &str) -> Option<usize>;
    fn link_reg(&mut self, name: &str, to: usize) -> bool;
    fn exec(&mut self, t: f32, regs: &mut [f32]);
}

struct DoSin {
    amp:    usize,
    phase:  usize,
    vert:   usize,
    f:      usize,
    out:    usize,
}

impl DoSin {
    fn new() -> Self { DoSin { amp: 0, phase: 0, vert: 0, out: 0, f: 0 } }
}

impl DemOp for DoSin {
    fn alloc_regs(&self) -> usize { 5 }
    fn init_regs(&mut self, start_reg: usize, regs: &mut [f32]) {
        regs[start_reg]     = 1.0;
        regs[start_reg + 3] = 0.001;

        self.amp   = start_reg;
        self.phase = start_reg + 1;
        self.vert  = start_reg + 2;
        self.f     = start_reg + 3;
        self.out   = start_reg + 4;
    }

    fn get_reg(&mut self, name: &str) -> Option<usize> {
        match name {
            "amp"   => Some(self.amp),
            "phase" => Some(self.phase),
            "vert"  => Some(self.vert),
            "freq"  => Some(self.f),
            "out"   => Some(self.out),
            _       => None,
        }
    }

    fn link_reg(&mut self, name: &str, to: usize) -> bool {
        match name {
            "amp"   => { self.amp   = to; true },
            "phase" => { self.phase = to; true },
            "vert"  => { self.vert  = to; true },
            "freq"  => { self.f     = to; true },
            "out"   => { self.out   = to; true },
            _       => false,
        }
    }

    fn exec(&mut self, t: f32, regs: &mut [f32]) {
        let a = regs[self.amp];
        let p = regs[self.phase];
        let v = regs[self.vert];
        let f = regs[self.f];
        regs[self.out] = a * (((f * t) + p).sin() + v);
        //d// println!("OUT: {}, {}", regs[self.out], self.out);
    }
}

struct Simulator {
    regs:   Vec<f32>,
    ops:    Vec<Box<dyn DemOp>>,
}

struct ClContext {
    sim: Simulator,
}

impl ClContext {
    fn new_op(&mut self, idx: usize, t: &str) -> Option<usize> {
        let sim = &mut self.sim;
        let mut o : Box<dyn DemOp> = match t {
            "sin" => { Box::new(DoSin::new()) },
            _     => { return None; },
        };

        let new_start_reg = sim.regs.len();
        sim.regs.resize(sim.regs.len() + o.alloc_regs(), 0.0);
        o.init_regs(new_start_reg, &mut sim.regs[..]);
        let out_reg = o.get_reg("out");

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
}

pub fn main() -> Result<(), String> {
    use wlambda::prelude::create_wlamba_prelude;
    use wlambda::vval::{Env};


    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let clctx = Rc::new(RefCell::new(ClContext {
        sim: Simulator {
            ops: Vec::new(),
            regs: Vec::new(),
        },
    }));

    let global_env = create_wlamba_prelude();

    global_env.borrow_mut().add_func(
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

    global_env.borrow_mut().add_func(
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

    let mut ctx = wlambda::compiler::EvalContext::new_with_user(global_env, clctx.clone());
    ctx.eval_file(&std::env::args().nth(1).unwrap_or("in.wl".to_string())).unwrap();

    let draw_cb = ctx.get_global_var("draw");
    if draw_cb.is_none() {
        panic!("script did not setup a global draw() function!");
    }
    let draw_cb = draw_cb.unwrap();
    if !draw_cb.is_fun() {
        panic!("script did not setup a global draw() function!");
    }

    let mut event_pump = sdl_context.event_pump()?;

//    let ttf_ctx = sdl2::ttf::init().map_err(|e| e.to_string())?;

//    let mut font = ttf_ctx.load_font("DejaVuSansMono.ttf", 14).map_err(|e| e.to_string())?;
////    font.set_style(sdl2::ttf::FontStyle::BOLD | sdl2::ttf::FontStyle::UNDERLINE);
//    font.set_hinting(sdl2::ttf::Hinting::Normal);
////    font.set_outline_width(0.1);
//    font.set_kerning(true);

    let mut start_time = Instant::now();
    let mut last_frame = Instant::now();
    let mut is_first = true;
    'running: loop {
        let event = event_pump.wait_event_timeout(16);
        if let Some(event) = event {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
//                Event::KeyDown { keycode: Some(Keycode::H), .. } => {
//                },
//                Event::KeyDown { keycode: Some(Keycode::J), .. } => {
//                },
//                Event::KeyDown { keycode: Some(Keycode::K), .. } => {
//                },
//                Event::KeyDown { keycode: Some(Keycode::L), .. } => {
//                },
//                Event::KeyDown { keycode: Some(Keycode::U), .. } => {
//                },
//                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
//                },
//                Event::KeyDown { keycode: Some(Keycode::X), .. } => {
//                },
//                Event::KeyDown { keycode: Some(Keycode::Y), .. } => {
//                },
//                Event::MouseButtonDown { x: x, y: y, .. } => {
//                },
//                Event::TextInput { text: text, .. } => {
//                    println!("TEXT: {}", text);
//                },
//                Event::MouseWheel { y: y, direction: dir, .. } => {
//                    match dir {
//                        sdl2::mouse::MouseWheelDirection::Normal => {
//                            println!("DIR NORMAL");
//                        },
//                        sdl2::mouse::MouseWheelDirection::Flipped => {
//                            println!("DIR FLOP");
//                        },
//                        _ => {}
//                    }
//                },
//                Event::Window { win_event: w, timestamp: _, window_id: _ } => {
//                    match w {
//                        WindowEvent::Resized(w, h) => { },
//                        WindowEvent::SizeChanged(w, h) => { },
//                        WindowEvent::FocusGained => { },
//                        WindowEvent::FocusLost => { },
//                        _ => {}
//                    }
//                },
                _ => {}
            }
        }

        let frame_time = last_frame.elapsed().as_millis();
        //println!("FO {},{},{}", frame_time, is_first, force_redraw);

        if is_first || frame_time >= 16 {
            extern crate palette;
            use palette::{Rgb};
//            use palette::pixel::Srgb;

            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();

            let now_time = start_time.elapsed().as_millis();
            let r = ctx.call(&draw_cb, &vec![VVal::Int(now_time as i64)]).unwrap();
            let hue : palette::Hsv = palette::Hsv::new((r.f() as f32).into(), 1.0, 1.0);
            let rc : Rgb = hue.into();

            clctx.borrow_mut().exec(now_time as f32);

            canvas.set_draw_color(Color::RGB(
                (rc.red * 255.0) as u8,
                (rc.green * 255.0) as u8,
                (rc.blue * 255.0) as u8));
            canvas.fill_rect(Rect::new(10, 10, 400, 400));
//            r.at(0).i();
            canvas.present();
            last_frame = Instant::now();
        }

        is_first = false;
    }

    Ok(())
}
