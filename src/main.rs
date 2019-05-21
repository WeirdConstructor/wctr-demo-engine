use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
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
    pub base:       std::path::PathBuf,
    pub paths:      std::vec::Vec<PathRecord>,
}

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

        for e in fs::read_dir(".")? {
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
            base:   path.to_path_buf(),
            paths:  sheet_paths,
        })
    }
}


pub enum DrawLineAttr {
    Dir,
    File,
    Special,
}

pub struct DrawLine {
    text:   String,
    time:   std::time::SystemTime,
    attr:   DrawLineAttr,
}

pub trait FmPage {
    fn len(&self) -> usize;
    fn as_draw_line(&self) -> DrawLine;
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

const NORM_BG_COLOR  = Color::RGB( 19,  19,  19);
const NORM_BG2_COLOR = Color::RGB( 38,  38,  38);
const NORM_BG3_COLOR = Color::RGB( 51,  51,  51);
const NORM_FG_COLOR  = Color::RGB(229, 229, 229);
const CURS_BG_COLOR  = Color::RGB(144, 238, 144);
const CURS_BG_COLOR  = Color::RGB(  0,   0,   0);
const HIGH_BG_COLOR  = Color::RGB(255,   0,   0);
const HIGH_FG_COLOR  = Color::RGB(  0,   0,   0);
const SLCT_BG_COLOR  = Color::RGB(169, 169, 169);
const SLCT_FG_COLOR  = Color::RGB(  0,   0,   0);
const DIVIDER_COLOR  = Color::RGB( 34,  69,  34);
const SCRIND_COLOR   = Color::RGB( 96, 255,  96);
const DIR_FG_COLOR   = Color::RGB( 64, 255, 255);
const LNK_FG_COLOR   = Color::RGB(255, 128, 255);

fn draw_text(font: &mut sdl2::ttf::Font, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, x: i32, y: i32, txt: &str) {
    let txt_crt = canvas.texture_creator();

    let sf = font.render(txt).blended(Color::RGBA(0, 0, 0, 255)).map_err(|e| e.to_string()).unwrap();
    let mut txt = txt_crt.create_texture_from_surface(&sf).map_err(|e| e.to_string()).unwrap();
    let tq = txt.query();


//    txt.set_color_mod(255, 0, 0);
    canvas.copy(&txt, None, Some(Rect::new(x, y, tq.width, tq.height))).map_err(|e| e.to_string()).unwrap();
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

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;


    let ttf_ctx = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut font = ttf_ctx.load_font("DejaVuSansMono.ttf", 10).map_err(|e| e.to_string())?;
//    font.set_style(sdl2::ttf::FontStyle::BOLD | sdl2::ttf::FontStyle::UNDERLINE);
    font.set_hinting(sdl2::ttf::Hinting::Mono);
    font.set_kerning(false);

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

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        let mut y = 0i32;
        for e in fs::read_dir(".").unwrap() {
            let pth = e.unwrap().path();
            if pth.is_dir() {
                draw_text(&mut font, &mut canvas, 0, y, &format!("D {}", pth.file_name().unwrap().to_string_lossy()));
            } else if pth.is_file() {
                draw_text(&mut font, &mut canvas, 0, y, &format!("f {}", pth.file_name().unwrap().to_string_lossy()));
            } else {
                draw_text(&mut font, &mut canvas, 0, y, &format!("? {}", pth.file_name().unwrap().to_string_lossy()));
            }
//            let dir = entry?;

            y += font.height();
        }

        canvas.present();

        let frame_time = last_frame.elapsed().as_millis();
        let last_wait_time = 16 - (frame_time as i64);

        if last_wait_time > 0 {
            ::std::thread::sleep(Duration::from_millis(last_wait_time as u64));
        }
    }

    Ok(())
}
