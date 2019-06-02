use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::rect::Point;
//use sdl2::sys::Point;
use chrono::offset::Utc;
use chrono::DateTime;
//use std::borrow::BorrowMut;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Instant};
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

#[derive(Debug)]
pub struct RenderFeedback {
    recent_line_count: usize,
    row_offset:        usize,
    start_rows:        (i32, i32),
    row_height:        i32,
    end_rows:          (i32, i32),
}

struct PathSheet {
    pub base:               std::path::PathBuf,
    pub paths:              std::vec::Vec<PathRecord>,
    pub paths_dirty:        bool,
    pub state_dirty:        bool,
    pub cursor_idx:         usize,
    pub scroll_offset:      usize,
    pub selection:          std::collections::HashSet<usize>,
    pub highlight:          std::collections::HashSet<usize>,
    pub render_feedback:    RenderFeedback,
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
            scroll_offset:  0,
            render_feedback: RenderFeedback {
                recent_line_count: 0,
                row_offset:        0,
                start_rows:        (0, 0),
                row_height:        0,
                end_rows:          (0, 0),
            },
            selection:      std::collections::HashSet::new(),
            highlight:      std::collections::HashSet::new(),
            paths_dirty:    false,
            state_dirty:    false,
        })
    }
}

pub enum Style {
    Default,
    Dir,
    File,
    Special,
}

pub struct StyleString {
    pub text: String,
    pub style: Style,
}

enum ColumnSizing {
    TextWidth(String),
    ExpandFract(i32),
}

pub struct Column {
    head:          String,
    rows:          std::vec::Vec<StyleString>,
    size:          ColumnSizing,
    calc_size:     Option<i32>,
}

pub struct Table {
    title:      String,
    columns:    std::vec::Vec<Column>,
    row_gap:    u32,
    col_gap:    u32,
}

pub enum PageControl {
    Refresh,
    Back,
    Access,
    CursorDown,
    CursorUp,
    Click((i32, i32)),
    Scroll(i32),
}

pub trait FmPage {
    fn len(&self) -> usize;
    fn as_draw_page(&self) -> Table;
    fn get_scroll_offs(&self) -> usize;
    fn do_control(&mut self, ctrl: PageControl);
    fn is_cursor_idx(&self, idx: usize) -> bool;
    fn is_selected(&self, idx: usize) -> bool;
    fn is_highlighted(&self, idx: usize) -> bool;
    fn needs_repage(&self) -> bool;
    fn needs_redraw(&self) -> bool;

    fn sort_by_column(&mut self, col_idx: usize);

    fn set_render_feedback(&mut self, fb: RenderFeedback);
}

const SCROLL_PADDING : usize = 5;

impl FmPage for PathSheet {
    fn len(&self) -> usize { self.paths.len() }
    fn get_scroll_offs(&self) -> usize { self.scroll_offset }
    fn is_cursor_idx(&self, idx: usize) -> bool { self.cursor_idx == idx }
    fn is_selected(&self, idx: usize) -> bool { self.selection.get(&idx).is_some() }
    fn is_highlighted(&self, idx: usize) -> bool { self.highlight.get(&idx).is_some() }
    fn needs_repage(&self) -> bool { self.paths_dirty }
    fn needs_redraw(&self) -> bool { self.state_dirty }

    fn sort_by_column(&mut self, col_idx: usize) {
        if col_idx == 0 {
            self.paths.sort_by(|a, b| {
                let s1 = String::from(
                    a.path.file_name()
                    .unwrap_or(std::ffi::OsStr::new(""))
                    .to_string_lossy()).to_lowercase();
                let s2 = String::from(
                    b.path.file_name()
                    .unwrap_or(std::ffi::OsStr::new(""))
                    .to_string_lossy()).to_lowercase();

                if let PathRecordType::Dir = a.path_type {
                    if let PathRecordType::Dir = b.path_type {
                        s1.partial_cmp(&s2).unwrap()
                    } else {
                        std::cmp::Ordering::Less
                    }
                } else {
                    if let PathRecordType::Dir = b.path_type {
                        std::cmp::Ordering::Greater
                    } else {
                        s1.partial_cmp(&s2).unwrap()
                    }
                }
            });
        } else if col_idx == 1 {
            self.paths.sort_by(|a, b| a.mtime.partial_cmp(&b.mtime).unwrap());
        } else if col_idx == 2 {
            self.paths.sort_by(|a, b| a.size.partial_cmp(&b.size).unwrap());
        }

        self.paths_dirty = true;
    }

    fn set_render_feedback(&mut self, fb: RenderFeedback) {
        self.render_feedback = fb;
    }

    fn do_control(&mut self, ctrl: PageControl) {
        match ctrl {
            PageControl::CursorDown => {
                self.cursor_idx += 1;
            },
            PageControl::CursorUp => {
                if self.cursor_idx > 0 {
                    self.cursor_idx -= 1;
                }
            },
            PageControl::Click((x, y)) => {
                println!("CLICK {}, {} | {:?}", x, y, self.render_feedback);
                let x1 = self.render_feedback.start_rows.0;
                let x2 = self.render_feedback.end_rows.0;
                let y1 = self.render_feedback.start_rows.1;
                let y2 = self.render_feedback.end_rows.1;

                if !(x >= x1 && x <= x2 && y >= y1 && y <= y2) {
                    return;
                }

                let y = y - y1;
                let row = y / self.render_feedback.row_height;
                self.cursor_idx = self.render_feedback.row_offset + row as usize;
            },
            PageControl::Scroll(amount) => {
                println!("SCROLL {}", amount);
                let amount = amount * SCROLL_PADDING as i32;
                if amount < 0 && self.scroll_offset < (-amount) as usize {
                    self.scroll_offset = 0;

                } else if amount < 0 {
                    self.scroll_offset -= (-amount) as usize;

                } else {
                    self.scroll_offset += amount as usize;
                }

                if self.len() <= self.render_feedback.recent_line_count {
                    self.scroll_offset = 0;
                } else {
                    if self.scroll_offset > (self.len() - self.render_feedback.recent_line_count) {
                        self.scroll_offset = self.len() - self.render_feedback.recent_line_count;
                    }
                }

                return;
            },
            _ => {},
        }

        println!("CURSOR CTRL {} len:{}, offs:{} disp:{}",
                 self.cursor_idx,
                 self.len(),
                 self.scroll_offset,
                 self.render_feedback.recent_line_count);

        if self.cursor_idx >= self.len() {
            self.cursor_idx = if self.len() > 0 { self.len() - 1 } else { 0 };
        }

        let recent_linecnt = self.render_feedback.recent_line_count;

        if recent_linecnt <= 2 * SCROLL_PADDING {
            if self.cursor_idx > 0 {
                self.scroll_offset = self.cursor_idx - 1;
            } else {
                self.scroll_offset = self.cursor_idx;
            }
        } else {
            if self.cursor_idx < (self.scroll_offset + SCROLL_PADDING) {
                let diff = (self.scroll_offset + SCROLL_PADDING) - self.cursor_idx;
                if self.scroll_offset > diff {
                    self.scroll_offset -= diff;
                } else {
                    self.scroll_offset = 0;
                }

            } else if (self.cursor_idx + SCROLL_PADDING + 1) > (self.scroll_offset + recent_linecnt) {
                self.scroll_offset += (self.cursor_idx + SCROLL_PADDING + 1) - (self.scroll_offset + recent_linecnt);
            }

            if (self.scroll_offset + recent_linecnt) > self.len() {
                if self.len() < recent_linecnt {
                    self.scroll_offset = 0;
                } else {
                    self.scroll_offset = self.len() - recent_linecnt;
                }
            }
        }

        println!("END CURSOR CTRL {} len:{}, offs:{} disp:{}", self.cursor_idx, self.len(), self.scroll_offset, recent_linecnt);
    }

    fn as_draw_page(&self) -> Table {
        Table {
            title: String::from(self.base.to_string_lossy()),
            row_gap: 2,
            col_gap: 4,
            columns: vec![
                Column {
                    head: String::from("name"),
                    size: ColumnSizing::ExpandFract(1),
                    calc_size: None,
                    rows: self.paths.iter().map(|p| {
                        let mut path_postfix = String::from("");
                        if let PathRecordType::Dir = p.path_type {
                            path_postfix = std::path::MAIN_SEPARATOR.to_string();
                        };

                        StyleString {
                            text: String::from(p.path.file_name()
                                                .unwrap_or(std::ffi::OsStr::new(""))
                                                .to_string_lossy()) + &path_postfix,
                            style: match p.path_type {
                                PathRecordType::File    => Style::File,
                                PathRecordType::Dir     => Style::Dir,
                                PathRecordType::SymLink => Style::Special,
                            }
                        }
                    }).collect(),
                },
                Column {
                    head: String::from("time"),
                    size: ColumnSizing::TextWidth(String::from("MMMM-MM-MM MM:MM:MM")),
                    calc_size: None,
                    rows: self.paths.iter().map(|p| {
                        let dt : DateTime<Utc> = p.mtime.into();
                        StyleString { text: format!("{}", dt.format("%Y-%m-%d %H:%M:%S")), style: Style::Default }
                    }).collect(),
                },
                Column {
                    head: String::from("size"),
                    size: ColumnSizing::TextWidth(String::from("MMMMMM")),
                    calc_size: None,
                    rows: self.paths.iter().map(|_p| {
                        StyleString { text: String::from("{???}"), style: Style::Default }
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

pub enum FileManagerSide {
    Left,
    Right,
}

pub struct FileManager<'a, 'b> {
    draw_state:     DrawState<'a, 'b>,
    left:           std::vec::Vec<Page>,
    right:          std::vec::Vec<Page>,
    active_side:    FileManagerSide,
}

enum PanePos {
    LeftTab,
    RightTab,
}

impl<'a, 'b> FileManager<'a, 'b> {
    fn open_path_in(&mut self, path: &std::path::Path, pos: PanePos) {
        let ps = PathSheet::read(path).expect("No broken paths please");
        let r = Rc::new(ps);
        let mut pg = Page {
            fm_page: r,
            cache: None,
        };
        Rc::get_mut(&mut pg.fm_page).unwrap().sort_by_column(0);
        match pos {
            PanePos::LeftTab  => self.left.push(pg),
            PanePos::RightTab => self.right.push(pg),
        }
    }

    fn process_page_control(&mut self, ctrl: PageControl) {
        match self.active_side {
            FileManagerSide::Left => {
                if self.left.is_empty() { return; }
                Rc::get_mut(&mut self.left.get_mut(0).unwrap().fm_page).unwrap()
                    .do_control(ctrl);
            },
            FileManagerSide::Right => {
                if self.right.is_empty() { return; }
//                Rc::get_mut(self.right.get_mut(0).unwrap().fm_page).unwrap().do_control(ctrl);
            },
        };
    }

    fn handle_resize(&mut self) {
        if !self.left.is_empty() {
            let pg : &mut Page = self.left.get_mut(0).unwrap();
            Rc::get_mut(&mut pg.fm_page).unwrap().do_control(PageControl::Refresh);
        }

        if !self.right.is_empty() {
            let pg : &mut Page = self.right.get_mut(0).unwrap();
            Rc::get_mut(&mut pg.fm_page).unwrap().do_control(PageControl::Refresh);
        }
    }

    fn redraw(&mut self) {
        if !self.left.is_empty() {
            let pg : &mut Page = self.left.get_mut(0).unwrap();

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

            let has_focus : bool =
                0 <
                self.draw_state.canvas.window().window_flags()
                & (  (sdl2::sys::SDL_WindowFlags::SDL_WINDOW_INPUT_FOCUS as u32)
                   | (sdl2::sys::SDL_WindowFlags::SDL_WINDOW_MOUSE_FOCUS as u32));

            if pg.cache.is_some() {
                let render_feedback =
                    self.draw_state.draw_table(
                        pg,
                        2, 0,
                        half_width as i32 - 2,
                        win_size.1 as i32,
                        has_focus);
                Rc::get_mut(&mut pg.fm_page)
                    .unwrap()
                    .set_render_feedback(render_feedback);
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
const CURS_BG_COLOR  : Color = Color { r: 144, g: 238, b: 144, a: 0xff };
const CURS_FG_COLOR  : Color = Color { r:   0, g:   0, b:   0, a: 0xff };
const HIGH_BG_COLOR  : Color = Color { r: 255, g:   0, b:   0, a: 0xff };
const HIGH_FG_COLOR  : Color = Color { r:   0, g:   0, b:   0, a: 0xff };
const SLCT_BG_COLOR  : Color = Color { r: 169, g: 169, b: 169, a: 0xff };
const SLCT_FG_COLOR  : Color = Color { r:   0, g:   0, b:   0, a: 0xff };
const SCRIND_COLOR   : Color = Color { r:  96, g: 255, b:  96, a: 0xff };
const DIR_FG_COLOR   : Color = Color { r:  64, g: 255, b: 255, a: 0xff };
const LNK_FG_COLOR   : Color = Color { r: 255, g: 128, b: 255, a: 0xff };
const DIVIDER_COLOR  : Color = Color { r:  34, g:  69, b:  34, a: 0xff };

const MIN_EXPAND_WIDTH : i32 = 50;

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

        if expand_rest_width < MIN_EXPAND_WIDTH {
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
        &mut self,
        pg: &mut Page,
        x_offs: i32,
        y_offs: i32,
        table_width: i32,
        table_height: i32,
        has_focus: bool) -> RenderFeedback {

        let table = pg.cache.as_mut().unwrap();

        self.calc_column_text_widths(table);
        let cols = self.calc_column_width(table, table_width, 0);

        let row_height = self.font.borrow().height() + table.row_gap as i32;

        let mut x = x_offs;
        for width_and_col in cols.iter().enumerate().zip(table.columns.iter()) {
            let col_idx = (width_and_col.0).0;
            let width   = (width_and_col.0).1;
            let column  = width_and_col.1;
            //d// println!("COL {}, w: {}, h: {}", col_idx, width, column.head);

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

            for (row_idx, row) in column.rows.iter()
                                    .enumerate()
                                    .skip(pg.fm_page.get_scroll_offs()) {

                if (y - y_offs) + row_height > table_height {
                    break;
                }

                let mut fg_color = match row.style {
                    Style::Dir     => DIR_FG_COLOR,
                    Style::Special => LNK_FG_COLOR,
                    _              => NORM_FG_COLOR,
                };

                if row_idx % 2 == 0 {
                    if col_idx % 2 == 0 {
                        self.canvas.set_draw_color(NORM_BG_COLOR);
                    } else {
                        self.canvas.set_draw_color(NORM_BG2_COLOR);
                    }
                } else {
                    if col_idx % 2 == 0 {
                        self.canvas.set_draw_color(NORM_BG2_COLOR);
                    } else {
                        self.canvas.set_draw_color(NORM_BG3_COLOR);
                    }
                }

                if has_focus && pg.fm_page.is_cursor_idx(row_idx) {
                    self.canvas.set_draw_color(CURS_BG_COLOR);
                    fg_color = CURS_FG_COLOR;

                } else if pg.fm_page.is_selected(row_idx) {
                    self.canvas.set_draw_color(SLCT_BG_COLOR);
                    fg_color = SLCT_BG_COLOR;
                }

                self.canvas.fill_rect(Rect::new(x, y, *width as u32, row_height as u32));

                draw_text(
                    &mut self.font.borrow_mut(),
                    fg_color,
                    &mut self.canvas,
                    x,
                    y,
                    *width - table.col_gap as i32,
                    &row.text);
                y += row_height;
            }

            x += width;
            //d// println!("X= {}", x);
        }

        let line_count = ((table_height - row_height) / row_height) as i32;
        RenderFeedback {
            // substract 1 row_height for title bar
            recent_line_count: line_count as usize,
            row_offset: pg.fm_page.get_scroll_offs(),
            start_rows: (x_offs,
                         y_offs + row_height),
            row_height,
            end_rows:   (x_offs + table_width,
                         y_offs + row_height + line_count as i32 * row_height),
        }
    }
}

fn draw_text(font: &mut sdl2::ttf::Font, color: Color, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, x: i32, y: i32, max_w: i32, txt: &str) {
    let txt_crt = canvas.texture_creator();

    let sf = font.render(txt).blended(color).map_err(|e| e.to_string()).unwrap();
    let txt = txt_crt.create_texture_from_surface(&sf).map_err(|e| e.to_string()).unwrap();
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
        active_side: FileManagerSide::Left,
        left: Vec::new(),
        right: Vec::new(),
    };

    let pth = std::path::Path::new(".");
    fm.open_path_in(pth, PanePos::LeftTab);

    let mut last_frame = Instant::now();
    let mut is_first = true;
    'running: loop {
        let mut force_redraw = false;
        let event = event_pump.wait_event_timeout(1000);
        if let Some(event) = event {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::H), .. } => {
                    fm.process_page_control(PageControl::Back);
                },
                Event::KeyDown { keycode: Some(Keycode::J), .. } => {
                    fm.process_page_control(PageControl::CursorDown);
                },
                Event::KeyDown { keycode: Some(Keycode::K), .. } => {
                    fm.process_page_control(PageControl::CursorUp);
                },
                Event::KeyDown { keycode: Some(Keycode::L), .. } => {
                    fm.process_page_control(PageControl::Access);
                },
                Event::MouseButtonDown { x: x, y: y, .. } => {
                    fm.process_page_control(PageControl::Click((x, y)));
                },
                Event::MouseWheel { y: y, direction: dir, .. } => {
                    match dir {
                        sdl2::mouse::MouseWheelDirection::Normal => {
                            fm.process_page_control(PageControl::Scroll(-y));
                            println!("DIR NORMAL");
                        },
                        sdl2::mouse::MouseWheelDirection::Flipped => {
                            fm.process_page_control(PageControl::Scroll(y));
                            println!("DIR FLOP");
                        },
                        _ => {}
                    }
                },
                Event::Window { win_event: w, timestamp: _, window_id: _ } => {
                    match w {
                        WindowEvent::Resized(w, h) => {
                            println!("XHX {},{}", w, h);
                            fm.handle_resize();
                            force_redraw = true;
                        },
                        WindowEvent::SizeChanged(w, h) => {
                            println!("XHXSC {},{}", w, h);
                            fm.handle_resize();
                            force_redraw = true;
                        },
                        WindowEvent::FocusGained => {
                            force_redraw = true;
                        },
                        WindowEvent::FocusLost => {
                            force_redraw = true;
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        let frame_time = last_frame.elapsed().as_millis();
        println!("FO {},{},{}", frame_time, is_first, force_redraw);

        if is_first || force_redraw || frame_time >= 16 {
            fm.draw_state.clear();
            fm.redraw();
            fm.draw_state.done();
            last_frame = Instant::now();
        }

        is_first = false;
    }

    Ok(())
}
