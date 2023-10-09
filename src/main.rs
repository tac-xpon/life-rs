use piston_window::{
    PistonWindow,
    WindowSettings,
    OpenGL,
    EventLoop,
};
use sdl2_window::Sdl2Window;
mod life_cell;
use life_cell::*;

mod direction;
use direction::*;

mod bgchar_data;
mod bgpal_data;

mod wait_and_update;

use bgsp_lib2::{
    bgsp_common::*,
    bg_plane::*,
};

pub type GameWindow = PistonWindow<Sdl2Window>;

pub struct DisplayInfo {
    pub full_screen: bool,
    pub vm_rect_size: (i32, i32),
    pub rotation: Direction,
    pub pixel_scale: i32,
    pub margin: i32,
    pub f_count: i32,
}

const WORLD_SIZE: (usize, usize) = (128, 128);

const FULL_SCREEN: bool = false;
const VM_RECT_SIZE: (i32, i32) = (128 * PATTERN_SIZE as i32, 120 * PATTERN_SIZE as i32);
const ROTATION: Direction = Direction::Normal;
const PIXEL_SCALE: i32 = 1;
const WINDOW_MARGIN: i32 = 0;
const BG0_RECT_SIZE: (i32, i32) = (128, 128);
const BG1_RECT_SIZE: (i32, i32) = (128, 128);

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut window: GameWindow;
    let mut display_info = {
        let full_screen = FULL_SCREEN;
        let vm_rect_size = VM_RECT_SIZE;
        let rotation = ROTATION;
        let pixel_scale = PIXEL_SCALE;
        let margin = WINDOW_MARGIN;
        let view_rect = {
            let (width, height) = (vm_rect_size.0 * pixel_scale, vm_rect_size.1 * pixel_scale);
            match rotation {
                Direction::Up    | Direction::Down => (width, height),
                Direction::Right | Direction::Left => (height, width),
            }
        };

        window = {
            const OPENGL_VER: OpenGL = OpenGL::V3_2;
            let window_title = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            let window_rect_size = if full_screen {
                [8192, 8192]
            } else {
                [(view_rect.0 + margin * 2) as u32, (view_rect.1 + margin * 2) as u32]
            };
            let window_setting = WindowSettings::new(&window_title, window_rect_size)
                .samples(0)
                .fullscreen(full_screen)
                .exit_on_esc(true)
                .graphics_api(OpenGL::V3_2)
                .vsync(true)
                .resizable(false)
                .decorated(true)
                .controllers(true)
            ;
            let sdl2_window = Sdl2Window::with_subsystem(
                video_subsystem,
                &window_setting,
            ).unwrap();
            PistonWindow::new(OPENGL_VER, 0, sdl2_window)
        };
        window.set_max_fps(120);
        window.set_ups(60);
        window.set_ups_reset(0);
        window.set_swap_buffers(true);
        window.set_bench_mode(false);
        window.set_lazy(false);
        DisplayInfo {
            full_screen,
            vm_rect_size,
            rotation,
            pixel_scale,
            margin,
            f_count: 0,
        }
    };

    let mut bg = {
        let mut bg0 = BgPlane::new(
            BG0_RECT_SIZE,
            VM_RECT_SIZE,
            &bgchar_data::BG_CHARS,
            &bgpal_data::COLOR_TBL,
            display_info.pixel_scale,
        );
        bg0.set_base_symmetry(BgSymmetry::Normal);

        let mut bg1 = BgPlane::new(
            BG1_RECT_SIZE,
            VM_RECT_SIZE,
            &bgchar_data::BG_CHARS,
            &bgpal_data::COLOR_TBL,
            display_info.pixel_scale,
        );
        bg1.set_base_symmetry(BgSymmetry::Normal);
        (bg0, bg1)
    };

    if display_info.full_screen { sdl_context.mouse().show_cursor(false) }

    let mut world = World::new(WORLD_SIZE);
    let mut lives = 0;
    {
        let (hx, hy) = (60, 62);
        lives += world.set_cell((hx + 1, hy + 0), CellState::Live);
        lives += world.set_cell((hx + 3, hy + 1), CellState::Live);
        lives += world.set_cell((hx + 0, hy + 2), CellState::Live);
        lives += world.set_cell((hx + 1, hy + 2), CellState::Live);
        lives += world.set_cell((hx + 4, hy + 2), CellState::Live);
        lives += world.set_cell((hx + 5, hy + 2), CellState::Live);
        lives += world.set_cell((hx + 6, hy + 2), CellState::Live);
    }

    let mut g_count = 0;
    'mail_loop: loop {
        if display_info.f_count % 4 == 0 {
            for y in 0..WORLD_SIZE.1 {
                for x in 0..WORLD_SIZE.0 {
                    let cell = world.read_cell((x, y));
                    bg.1.set_cur_pos(x as i32, y as i32)
                        .put_code(if cell.live == CellState::Live { '*' } else { ' ' })
                        .put_palette(1)
                    ;
                }
            }
            bg.0.set_cur_pos(1, 1)
                .put_string(&format!("Gen:{} Lives:{}  ", &g_count, &lives), Some(&CharAttributes::new(2, BgSymmetry::Normal)))
            ;
            lives += world.update_world();
            g_count += 1;
        }
        if wait_and_update::doing(&mut window, &mut bg, &mut display_info) { break 'mail_loop }
    }
    sdl_context.mouse().show_cursor(true);
}
