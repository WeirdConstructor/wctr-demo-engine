use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::rect::Point;
use chrono::offset::Utc;
use chrono::DateTime;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Instant, Duration};
use std::fs;

pub enum PathRecordType {
    File,
    Dir,
    SymLink,
}

pub struct PathRecord {
    pub path:       std::path::PathBuf,
    pub size:       u64,
    pub mtime:      std::time::SystemTime,
    pub path_type:  PathRecordType,
}

struct PathSheet {
    pub base:           std::path::PathBuf,
    pub paths:          std::vec::Vec<PathRecord>,
    pub paths_dirty:    bool,
    pub state_dirty:    bool,
    pub cursor_idx:     usize,
    pub selection:      std::collections::HashSet<usize>,
}

#[derive(Debug)]
enum FMError {
    IOError(std::io::Error),
}

impl std::convert::From<std::io::Error> for FMError {
    fn from(error: std::io::Error) -> Self {
        FMError::IOError(error)
    }
}

impl PathSheet {
    pub fn read(path: &std::path::Path) -> Result<PathSheet, FMError> {
        let mut sheet_paths = Vec::new();

        for e in fs::read_dir(path)? {
            let entry = e?;
            let path  = entry.path();
            let md    = path.symlink_metadata()?;
            let ft    = md.file_type();

            let pr = PathRecord {
                path,
                size:  md.len(),
                mtime: md.modified()?,
                path_type: if ft.is_symlink() {
                    PathRecordType::SymLink
                } else if ft.is_dir() {
                    PathRecordType::Dir
                } else {
                    PathRecordType::File
                },
            };

            sheet_paths.push(pr);
        }

        Ok(PathSheet {
            base:           path.to_path_buf(),
            paths:          sheet_paths,
            cursor_idx:     0,
            selection:      std::collections::HashSet::new(),
            paths_dirty:    false,
            state_dirty:    false,
        })
    }
}

pub enum DrawLineAttr {
    Dir,
    File,
    Special,
}

pub struct DrawLine {
    text:           String,
    time:           std::time::SystemTime,
    attr:           DrawLineAttr,
}

enum ColumnSizing {
    TextWidth(String),
    ExpandFract(i32),
}

pub struct Column {
    head:          String,
    rows:          std::vec::Vec<String>,
    size:          ColumnSizing,
    calc_size:     Option<i32>,
}

pub struct Table {
    title:      String,
    columns:    std::vec::Vec<Column>,
    row_gap:    u32,
    col_gap:    u32,
}

pub trait FmPage {
    fn len(&self) -> usize;
    fn as_draw_page(&self) -> Table;
    fn get_cursor_idx(&self) -> usize;
    fn is_selected(&self, idx: usize) -> bool;
    fn needs_repage(&self) -> bool;
    fn needs_redraw(&self) -> bool;
}

impl FmPage for PathSheet {
    fn len(&self) -> usize { self.paths.len() }
    fn get_cursor_idx(&self) -> usize { self.cursor_idx }
    fn is_selected(&self, idx: usize) -> bool { self.selection.get(&idx).is_some() }
    fn needs_repage(&self) -> bool { self.paths_dirty }
    fn needs_redraw(&self) -> bool { self.state_dirty }
    fn as_draw_page(&self) -> Table {
        Table {
            title: String::from(self.base.to_string_lossy()),
            row_gap: 2,
            col_gap: 10,
            columns: vec![
                Column {
                    head: String::from("name"),
                    size: ColumnSizing::ExpandFract(1),
                    calc_size: None,
                    rows: self.paths.iter().map(|p| {

                // Theme draus machen:
//                    attr: match p.path_type {
//                        PathRecordType::File    => DrawLineAttr::File,
//                        PathRecordType::Dir     => DrawLineAttr::Dir,
//                        PathRecordType::SymLink => DrawLineAttr::Special,
//                    }
                        String::from(p.path.file_name().unwrap_or(std::ffi::OsStr::new("")).to_string_lossy())
                    }).collect(),
                },
                Column {
                    head: String::from("time"),
                    size: ColumnSizing::TextWidth(String::from("MMMM-MM-MM MM:MM:MM")),
                    calc_size: None,
                    rows: self.paths.iter().map(|p| {
                        let dt : DateTime<Utc> = p.mtime.into();
                        format!("{}", dt.format("%Y-%m-%d %H:%M:%S"))
                    }).collect(),
                },
                Column {
                    head: String::from("size"),
                    size: ColumnSizing::TextWidth(String::from("MMMMMM")),
                    calc_size: None,
                    rows: self.paths.iter().map(|_p| {
                        String::from("{???}")
                    }).collect(),
                },
            ],
        }
    }
}

struct Page {
    fm_page:    Rc<dyn FmPage>,
    cache:      Option<Table>,
}

pub struct FileManager<'a, 'b> {
    draw_state:     DrawState<'a, 'b>,
    left:           std::vec::Vec<Page>,
    right:          std::vec::Vec<Page>,
}

enum PanePos {
    LeftTab,
    RightTab,
}

impl<'a, 'b> FileManager<'a, 'b> {
    fn open_path_in(&mut self, path: &std::path::Path, pos: PanePos) {
        let ps = PathSheet::read(path).expect("No broken paths please");
        let r = Rc::new(ps);
        let pg = Page {
            fm_page: r,
            cache: None,
        };
        match pos {
            PanePos::LeftTab  => self.left.push(pg),
            PanePos::RightTab => self.right.push(pg),
        }
    }

    fn redraw(&mut self) {
        if !self.left.is_empty() {
            let sl = self.left.as_mut_slice();
            let pg = &mut sl[sl.len() - 1];
            if pg.cache.is_none() || pg.fm_page.needs_repage() {
                pg.cache = Some(pg.fm_page.as_draw_page());
            }

            let win_size = self.draw_state.canvas.window().size();
            let half_width = win_size.0 / 2;
            self.draw_state.canvas.set_draw_color(NORM_BG_COLOR);
            self.draw_state.canvas.fill_rect(Rect::new(0, 0, win_size.0, win_size.1));
            self.draw_state.canvas.set_draw_color(DIVIDER_COLOR);
            self.draw_state.canvas.draw_line(
                Point::new(half_width as i32 + 1, 0),
                Point::new(half_width as i32 + 1, win_size.1 as i32));

            if pg.cache.is_some() {
                self.draw_state.draw_table(
                    pg.cache.as_mut().unwrap(),
                    0, 0,
                    half_width as i32);
            }
        }
    }
}

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

const NORM_BG_COLOR  : Color = Color { r:  19, g:  19, b:  19, a: 0xff };
const NORM_BG2_COLOR : Color = Color { r:  38, g:  38, b:  38, a: 0xff };
const NORM_BG3_COLOR : Color = Color { r:  51, g:  51, b:  51, a: 0xff };
const NORM_FG_COLOR  : Color = Color { r: 229, g: 229, b: 229, a: 0xff };
//const NORM_FG_COLOR  : Color = Color { r: 229, g: 229, b: 0, a: 0xff };
//const CURS_BG_COLOR  = Color::RGB(144, 238, 144);
//const CURS_BG_COLOR  = Color::RGB(  0,   0,   0);
//const HIGH_BG_COLOR  = Color::RGB(255,   0,   0);
//const HIGH_FG_COLOR  = Color::RGB(  0,   0,   0);
//const SLCT_BG_COLOR  = Color::RGB(169, 169, 169);
//const SLCT_FG_COLOR  = Color::RGB(  0,   0,   0);
const DIVIDER_COLOR : Color = Color { r: 34,  g: 69,  b: 34, a: 0xff };
//const SCRIND_COLOR   = Color::RGB( 96, 255,  96);
//const DIR_FG_COLOR   = Color::RGB( 64, 255, 255);
//const LNK_FG_COLOR   = Color::RGB(255, 128, 255);

struct DrawState<'a, 'b> {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    font: Rc<RefCell<sdl2::ttf::Font<'a, 'b>>>,
}

impl<'a, 'b> DrawState<'a, 'b> {
    fn clear(&mut self) {
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.canvas.clear();
    }

    fn done(&mut self) {
        self.canvas.present();
    }

    fn calc_column_text_widths(&mut self, table: &mut Table) {
        for col in table.columns.iter_mut() {
            if let ColumnSizing::TextWidth(txt) = &col.size {
                if col.calc_size.is_none() {
                    let tsize = self.font.borrow().size_of(&txt);
                    col.calc_size = Some(tsize.unwrap_or((0, 0)).0 as i32);
                } else {
                    col.calc_size = Some(0);
                }
            } else {
                col.calc_size = Some(0);
            }
        }
    }

    fn calc_column_width(&mut self, table: &Table, table_width: i32, skip_cols: u32) -> std::vec::Vec<i32> {
        if skip_cols >= table.columns.len() as u32 {
            return Vec::new();
        }

        let cols : std::vec::Vec<&Column> = table.columns.iter().rev().skip(skip_cols as usize).rev().collect();

        let fixed_width : i32 =
            cols.iter().map(|c| c.calc_size.unwrap() + table.col_gap as i32).sum();

        let expand_rest_width = table_width - fixed_width;

        if expand_rest_width < 0 {
            return self.calc_column_width(table, table_width, skip_cols + 1);
        }

        let fract_sum : u32 = cols.iter().map(|c|
            match c.size {
                ColumnSizing::ExpandFract(f) => f as u32,
                _ => 0u32,
            }).sum();

        cols.iter().map(|column|
            match column.size {
                ColumnSizing::TextWidth(_)   => column.calc_size.unwrap() + table.col_gap as i32,
                ColumnSizing::ExpandFract(f) => ((expand_rest_width * f) / fract_sum as i32) + table.col_gap as i32,
            }).collect()
    }

    fn draw_table(
        &mut self, table: &mut Table,
        x_offs: i32, y_offs: i32,
        table_width: i32) {
        println!("FO {}", table_width);

        self.calc_column_text_widths(table);
        let cols = self.calc_column_width(table, table_width, 0);

        let mut x = x_offs;
        for width_and_col in cols.iter().enumerate().zip(table.columns.iter()) {
            let row_idx = (width_and_col.0).0;
            let width   = (width_and_col.0).1;
            let column  = width_and_col.1;

            let row_height = self.font.borrow().height() + table.row_gap as i32;

            self.canvas.set_draw_color(NORM_BG_COLOR);
            self.canvas.fill_rect(Rect::new(x, y_offs, *width as u32, row_height as u32));
            draw_text(
                &mut self.font.borrow_mut(),
                NORM_FG_COLOR,
                &mut self.canvas,
                x,
                y_offs,
                *width - table.col_gap as i32,
                &column.head);

            self.canvas.set_draw_color(NORM_FG_COLOR);
            self.canvas.draw_line(
                Point::new(x,         y_offs + (row_height - table.row_gap as i32)),
                Point::new(x + width, y_offs + (row_height - table.row_gap as i32)));

            let mut y = y_offs + row_height;

            for (col_idx, row) in column.rows.iter().enumerate() {
                if col_idx % 2 == 0 {
                    if row_idx % 2 == 0 {
                        self.canvas.set_draw_color(NORM_BG_COLOR);
                    } else {
                        self.canvas.set_draw_color(NORM_BG2_COLOR);
                    }
                } else {
                    if row_idx % 2 == 0 {
                        self.canvas.set_draw_color(NORM_BG2_COLOR);
                    } else {
                        self.canvas.set_draw_color(NORM_BG3_COLOR);
                    }
                }
                self.canvas.fill_rect(Rect::new(x, y, *width as u32, row_height as u32));

                draw_text(
                    &mut self.font.borrow_mut(),
                    NORM_FG_COLOR,
                    &mut self.canvas,
                    x,
                    y,
                    *width - table.col_gap as i32,
                    &row);
                y += row_height;
            }

            x += width;
            println!("X= {}", x);
        }
    }
}

fn draw_text(font: &mut sdl2::ttf::Font, color: Color, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, x: i32, y: i32, max_w: i32, txt: &str) {
    let txt_crt = canvas.texture_creator();

    let sf = font.render(txt).blended(color).map_err(|e| e.to_string()).unwrap();
    let mut txt = txt_crt.create_texture_from_surface(&sf).map_err(|e| e.to_string()).unwrap();
    let tq = txt.query();

    let w : i32 = if max_w < (tq.width as i32) { max_w } else { tq.width as i32 };

//    txt.set_color_mod(255, 0, 0);
    canvas.copy(
        &txt,
        Some(Rect::new(0, 0, w as u32, tq.height)),
        Some(Rect::new(x, y, w as u32, tq.height))
    ).map_err(|e| e.to_string()).unwrap();
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .resizable()
//        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;


    let ttf_ctx = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut font = ttf_ctx.load_font("DejaVuSansMono.ttf", 14).map_err(|e| e.to_string())?;
//    font.set_style(sdl2::ttf::FontStyle::BOLD | sdl2::ttf::FontStyle::UNDERLINE);
    font.set_hinting(sdl2::ttf::Hinting::Normal);
//    font.set_outline_width(0.1);
    font.set_kerning(true);

    let mut fm = FileManager {
        draw_state: DrawState {
            canvas: canvas,
            font: Rc::new(RefCell::new(font)),
        },
        left: Vec::new(),
        right: Vec::new(),
    };

    'running: loop {
        let last_frame = Instant::now();
        let event = event_pump.wait_event();
        match event {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                break 'running
            },
            Event::Window { win_event: w, timestamp: _, window_id: _ } => {
                match w {
                    WindowEvent::Resized(w, h) => {
                        println!("XHX {},{}", w, h);
                    },
                    WindowEvent::SizeChanged(w, h) => {
                        println!("XHXSC {},{}", w, h);
                    },
                    _ => {}
                }
            },
            _ => {}
        }

        fm.draw_state.clear();

        let pth = std::path::Path::new(".");
//        let ps = PathSheet::read(pth).unwrap();
//        let dp = ps.as_draw_page();
        println!("WIN WIDTH: {}", fm.draw_state.canvas.window().size().0);
        fm.open_path_in(pth, PanePos::LeftTab);
        fm.redraw();
//        ds.draw_page(&dp);

//        let mut y = 0i32;
//        for e in fs::read_dir(".").unwrap() {
//            let pth = e.unwrap().path();
//            if pth.is_dir() {
//                draw_text(&mut font, &mut canvas, 0, y, &format!("D {}", pth.file_name().unwrap().to_string_lossy()));
//            } else if pth.is_file() {
//                draw_text(&mut font, &mut canvas, 0, y, &format!("f {}", pth.file_name().unwrap().to_string_lossy()));
//            } else {
//                draw_text(&mut font, &mut canvas, 0, y, &format!("? {}", pth.file_name().unwrap().to_string_lossy()));
//            }
////            let dir = entry?;
//
//            y += font.height();
//        }
//
        fm.draw_state.done();

        let frame_time = last_frame.elapsed().as_millis();
        let last_wait_time = 16 - (frame_time as i64);

        if last_wait_time > 0 {
            ::std::thread::sleep(Duration::from_millis(last_wait_time as u64));
        }
    }

    Ok(())
}
