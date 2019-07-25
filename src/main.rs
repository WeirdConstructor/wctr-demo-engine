//use sdl2::pixels::Color;
//use sdl2::event::Event;
////use sdl2::event::WindowEvent;
//use sdl2::keyboard::Keycode;
//use sdl2::rect::Rect;
//use sdl2::rect::Point;

mod turtle;
mod signals;
mod clcontext;
use clcontext::WLambdaCtx;
use turtle::{TurtleDrawing, ShapeRotation};

extern crate piston_window;
use piston_window::*;

use std::time::{Instant};


/* TODO:

    X use turtle state
    X implement turtle color
    - implement turtle vector direction
    - implement turtle line drawing
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

struct Painter<'a, T>
    where T: 'a + Graphics {
    ctx: &'a piston_window::Context,
    g: &'a mut T,
}

impl<'a, T> TurtleDrawing for Painter<'a, T>
    where T: 'a + Graphics {
    fn draw_line(&mut self, color: [f32; 4], _rot: ShapeRotation, from: [f32; 2], to: [f32; 2], thickness: f32) {
        let o = Ellipse::new(color);
        o.draw(
            ellipse::circle(from[0] as f64, from[1] as f64, thickness as f64),
            &self.ctx.draw_state, self.ctx.transform, self.g);
        o.draw(
            ellipse::circle(to[0] as f64, to[1] as f64, thickness as f64),
            &self.ctx.draw_state, self.ctx.transform, self.g);
        line_from_to(
            color, thickness as f64,
            [from[0] as f64, from[1] as f64],
            [to[0] as f64, to[1] as f64],
            self.ctx.transform, self.g);
    }

    fn draw_rect_fill(&mut self, color: [f32; 4], rot: ShapeRotation, pos: [f32; 2], size: [f32; 2]) {
        let rot = match rot {
            ShapeRotation::Center(a) => a,
            _ => 0.0,
        };
        rectangle(
            color,
            [(pos[0] - size[0] / 2.0) as f64,
             (pos[1] - size[1] / 2.0) as f64,
             size[0] as f64,
             size[1] as f64],
            self.ctx.transform.rot_rad(rot as f64),
            self.g);
    }
}

pub fn main() -> Result<(), String> {
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [640, 480])
        .graphics_api(opengl)
        .resizable(true)
        .vsync(true)
        .exit_on_esc(true).build().unwrap();

    let mut cnt = 0;
    let mut avg = 0;

    let mut wlctx = WLambdaCtx::new();
    wlctx.init();
    wlctx.load_script("in.wl");

    let start_time = Instant::now();
    while let Some(event) = window.next() {
        let ws = window.draw_size();

        window.draw_2d(&event, |mut context, graphics, _device| {
            clear([0.1; 4], graphics);

            let scale_size = 200.0 as f32;

            context.transform =
                context.transform.trans(
                    ws.width / 2.0 as f64,
                    ws.height / 2.0 as f64);

            let mut p = Painter {
                ctx: &context,
                g: graphics,
            };

            let b = Instant::now();

            let now_time = start_time.elapsed().as_millis();

            wlctx.one_step(now_time as i64, scale_size, &mut p);

            avg += b.elapsed().as_millis();
            cnt += 1;
            if cnt > 100 {
                println!("exec took {}", avg / cnt);
                cnt = 0;
                avg = 0;
            }

            // Turtle:
            //      color   (3 arbitrary OpIn regs: hsv)
            //      push        - state push (pos, direction, color)
            //      pop         - state pop
            //      move_to
            //      rot_rad (direction from last 2 movements)
            //      rot_deg (direction from last 2 movements)
            //      line_to
            //      line_walk
            //      rect_walk
            //      rect_to
            //      rect
            //      arc
            //      ellipse_walk
            //      ellipse_to
            //      ellipse

//            line_from_to(
//                [rc.red, rc.green, rc.blue,  1.0],
//                2.2 * r.f(),
//                [200.0, 150.0],
//                [300.0, 350.0],
//                context.transform,
//                graphics);

//            rectangle([rc.red, rc.green, rc.blue,  1.0],
//                      [100.0, 100.0, r.f() * 1.0, 100.0],
//                      context.transform.rot_deg(r.f()), graphics);
//            rectangle([1.0, 0.0, rc.blue, 1.0],
//                      [0.0, 0.0, r.f() * 1.0, 100.0],
//                      context.transform.trans(450.0, 150.0).rot_deg(r.f()).trans((-r.f() * 1.0) * 0.5, -50.0), graphics);
//            rectangle([rc.red, rc.green, 0.0, 1.0],
//                      [100.0, 400.0, r.f() * 1.0, 100.0],
//                      context.transform, graphics);
//            rectangle([1.0, rc.green, rc.blue, 1.0],
//                      [400.0, 400.0, r.f() * 1.0, 100.0],
//                      context.transform, graphics);
        });
    }


//    let mut start_time = Instant::now();
//    let mut last_frame = Instant::now();
//    let mut is_first = true;
//    'running: loop {
//        let event = event_pump.wait_event_timeout(16);
//        if let Some(event) = event {
//            match event {
//                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
//                    break 'running
//                },
////                Event::KeyDown { keycode: Some(Keycode::H), .. } => {
////                },
////                Event::KeyDown { keycode: Some(Keycode::J), .. } => {
////                },
////                Event::KeyDown { keycode: Some(Keycode::K), .. } => {
////                },
////                Event::KeyDown { keycode: Some(Keycode::L), .. } => {
////                },
////                Event::KeyDown { keycode: Some(Keycode::U), .. } => {
////                },
////                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
////                },
////                Event::KeyDown { keycode: Some(Keycode::X), .. } => {
////                },
////                Event::KeyDown { keycode: Some(Keycode::Y), .. } => {
////                },
////                Event::MouseButtonDown { x: x, y: y, .. } => {
////                },
////                Event::TextInput { text: text, .. } => {
////                    println!("TEXT: {}", text);
////                },
////                Event::MouseWheel { y: y, direction: dir, .. } => {
////                    match dir {
////                        sdl2::mouse::MouseWheelDirection::Normal => {
////                            println!("DIR NORMAL");
////                        },
////                        sdl2::mouse::MouseWheelDirection::Flipped => {
////                            println!("DIR FLOP");
////                        },
////                        _ => {}
////                    }
////                },
////                Event::Window { win_event: w, timestamp: _, window_id: _ } => {
////                    match w {
////                        WindowEvent::Resized(w, h) => { },
////                        WindowEvent::SizeChanged(w, h) => { },
////                        WindowEvent::FocusGained => { },
////                        WindowEvent::FocusLost => { },
////                        _ => {}
////                    }
////                },
//                _ => {}
//            }
//        }
//
//        let frame_time = last_frame.elapsed().as_millis();
//        //println!("FO {},{},{}", frame_time, is_first, force_redraw);
//
//        if is_first || frame_time >= 16 {
//            extern crate palette;
//            use palette::{Rgb};
////            use palette::pixel::Srgb;
//
//            canvas.set_draw_color(Color::RGB(0, 0, 0));
//            canvas.clear();
//
//            let now_time = start_time.elapsed().as_millis();
//            let r = ctx.call(&draw_cb, &vec![VVal::Int(now_time as i64)]).unwrap();
//            let hue : palette::Hsv = palette::Hsv::new((r.f() as f32).into(), 1.0, 1.0);
//            let rc : Rgb = hue.into();
//
//            clctx.borrow_mut().exec(now_time as f32);
//
//            canvas.set_draw_color(Color::RGB(
//                (rc.red * 255.0) as u8,
//                (rc.green * 255.0) as u8,
//                (rc.blue * 255.0) as u8));
//            canvas.fill_rect(Rect::new(10, 10, 400, 400));
////            r.at(0).i();
//            canvas.present();
//            last_frame = Instant::now();
//        }
//
//        is_first = false;
//    }

    Ok(())
}
