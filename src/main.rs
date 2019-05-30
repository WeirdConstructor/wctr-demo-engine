use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
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

pub struct Column {
    head:          String,
    rows:          std::vec::Vec<String>,
    min_width:     i32,
    fract_width:   i32,
}

pub struct Table {
    title:  String,
    columns:  std::vec::Vec<Column>,
}

pub trait FmPage {
    fn len(&self) -> usize;
    fn as_draw_page(&self) -> Table;
    fn is_cursor_here(&self, idx: usize) -> bool;
    fn is_selected(&self, idx: usize) -> bool;
    fn needs_repage(&self) -> bool;
    fn needs_redraw(&self) -> bool;
}

impl FmPage for PathSheet {
    fn len(&self) -> usize { self.paths.len() }
    fn is_cursor_here(&self, idx: usize) -> bool { idx == self.cursor_idx }
    fn is_selected(&self, idx: usize) -> bool { self.selection.get(&idx).is_some() }
    fn needs_repage(&self) -> bool { self.paths_dirty }
    fn needs_redraw(&self) -> bool { self.state_dirty }
    fn as_draw_page(&self) -> Table {
        Table {
            title: String::from(self.base.to_string_lossy()),
            columns: vec![
                Column {
                    head: String::from("name"),
                    min_width: 0,
                    fract_width: 5,
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
                    min_width: 0,
                    fract_width: 2,
                    rows: self.paths.iter().map(|_p| {
                        String::from("?time?")
                    }).collect(),
                },
                Column {
                    head: String::from("size"),
                    min_width: 0,
                    fract_width: 1,
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

            if pg.cache.is_some() {
                self.draw_state.draw_table(
                    pg.cache.as_mut().unwrap(), 0, 0, 300);
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

//const NORM_BG_COLOR  = Color::RGB( 19,  19,  19);
//const NORM_BG2_COLOR = Color::RGB( 38,  38,  38);
//const NORM_BG3_COLOR = Color::RGB( 51,  51,  51);
//const NORM_FG_COLOR  = Color::RGB(229, 229, 229);
//const CURS_BG_COLOR  = Color::RGB(144, 238, 144);
//const CURS_BG_COLOR  = Color::RGB(  0,   0,   0);
//const HIGH_BG_COLOR  = Color::RGB(255,   0,   0);
//const HIGH_FG_COLOR  = Color::RGB(  0,   0,   0);
//const SLCT_BG_COLOR  = Color::RGB(169, 169, 169);
//const SLCT_FG_COLOR  = Color::RGB(  0,   0,   0);
//const DIVIDER_COLOR  = Color::RGB( 34,  69,  34);
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

    fn draw_table(&mut self, table: &Table, x_offs: i32, y_offs: i32, width: i32) {
        let fracts : i32 = table.columns.iter().map(|c| c.fract_width).sum();

        let mut x = x_offs;
        for column in table.columns.iter() {
            let mut width = (width * column.fract_width) / fracts;
            if width < column.min_width { width = column.min_width; }

            draw_text(
                &mut self.font.borrow_mut(),
                &mut self.canvas,
                x,
                y_offs,
                width,
                &column.head);

            let mut y = y_offs + self.font.borrow().height();

            for row in column.rows.iter() {
                draw_text(
                    &mut self.font.borrow_mut(),
                    &mut self.canvas,
                    x,
                    y,
                    width,
                    &row);
                y += self.font.borrow().height();
            }

            x += width;
        }
    }
}

fn draw_text(font: &mut sdl2::ttf::Font, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, x: i32, y: i32, max_w: i32, txt: &str) {
    let txt_crt = canvas.texture_creator();

    let sf = font.render(txt).blended(Color::RGBA(0, 0, 0, 255)).map_err(|e| e.to_string()).unwrap();
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

    let mut font = ttf_ctx.load_font("DejaVuSansMono.ttf", 12).map_err(|e| e.to_string())?;
//    font.set_style(sdl2::ttf::FontStyle::BOLD | sdl2::ttf::FontStyle::UNDERLINE);
    font.set_hinting(sdl2::ttf::Hinting::Mono);
    font.set_kerning(false);

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

        let pth = std::path::Path::new("..");
//        let ps = PathSheet::read(pth).unwrap();
//        let dp = ps.as_draw_page();
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
