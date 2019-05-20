use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::{Instant, Duration};
use std::fs;

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

    let mut cnt = 0;
    let mut frame_time = 0;
    let mut last_wait_time = 0;
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

        cnt += 1;

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

        frame_time = last_frame.elapsed().as_millis();
        last_wait_time = 16 - (frame_time as i64);

        if last_wait_time > 0 {
            ::std::thread::sleep(Duration::from_millis(last_wait_time as u64));
        }
    }

    Ok(())
}
