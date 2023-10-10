use piston_window::{
    PistonWindow,
    WindowSettings,
    OpenGL,
    EventLoop,
    Key,
};
use sdl2_window::Sdl2Window;
use std::collections::BTreeMap;

mod life_cell;
use life_cell::*;

mod direction;
use direction::*;
mod input_role;
use input_role::*;

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

const WORLD_SIZE: (usize, usize) = (512, 512);

const FULL_SCREEN: bool = false;
const VM_RECT_SIZE: (i32, i32) = (64 * PATTERN_SIZE as i32, 60 * PATTERN_SIZE as i32);
const ROTATION: Direction = Direction::Normal;
const PIXEL_SCALE: i32 = 1;
const WINDOW_MARGIN: i32 = 0;
const BG0_RECT_SIZE: (i32, i32) = (128, 120);
const BG1_RECT_SIZE: (i32, i32) = (WORLD_SIZE.0 as i32, WORLD_SIZE.1 as i32);

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

    let mut keyboard_map: BTreeMap<piston_window::Key, Vec<_>> = BTreeMap::new();
    {
        let key_set_list = [
            (Key::D1,    InputRole::Progress1),
            (Key::D2,    InputRole::Progress2),
            (Key::D3,    InputRole::Progress4),
            (Key::D4,    InputRole::Progress8),
            (Key::P,     InputRole::Pause),
            (Key::O,     InputRole::OneTick),
            (Key::H,     InputRole::Home),
            (Key::Z,     InputRole::Button0),
            (Key::Space, InputRole::Button0),
            (Key::W,     InputRole::Up),
            (Key::D,     InputRole::Right),
            (Key::S,     InputRole::Down),
            (Key::A,     InputRole::Left),
            (Key::Up,    InputRole::Up),
            (Key::Right, InputRole::Right),
            (Key::Down,  InputRole::Down),
            (Key::Left,  InputRole::Left),
        ];
        for key_set in key_set_list {
            if let Some(role_list) = keyboard_map.get_mut(&key_set.0) {
                role_list.push(key_set.1);
            } else {
                keyboard_map.insert(key_set.0, vec![key_set.1]);
            }
        }
    }
    let mut input_role_state = InputRoleState::default();

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
    let mut g_span = 1;
    let mut renderd = false;
    let mut wait = 8;
    let mut pause = true;
    let mut one_tick = false;
    let mut view_pos = BgPos {x:0, y:0};
    input_role_state.clear_all();
    'mail_loop: loop {
        {
            let d = if input_role_state.get(InputRole::Button0).0 { 6 } else { 2 };
            if input_role_state.get(InputRole::Left).0 {
                view_pos.x -= d;
            }
            if input_role_state.get(InputRole::Right).0 {
                view_pos.x += d;
            }
            if input_role_state.get(InputRole::Up).0 {
                view_pos.y -= d;
            }
            if input_role_state.get(InputRole::Down).0 {
                view_pos.y += d;
            }
            if input_role_state.get(InputRole::Home).1 & 0b1111 == 0b1000 {
                view_pos.x = 0;
                view_pos.y = 0;
            }
            if input_role_state.get(InputRole::Progress1).1 & 0b1111 == 0b1000 {
                wait = 8;
                pause = false;
            }
            if input_role_state.get(InputRole::Progress2).1 & 0b1111 == 0b1000 {
                wait = 4;
                pause = false;
            }
            if input_role_state.get(InputRole::Progress4).1 & 0b1111 == 0b1000 {
                wait = 2;
                pause = false;
            }
            if input_role_state.get(InputRole::Progress8).1 & 0b1111 == 0b1000 {
                wait = 1;
                pause = false;
            }
            if input_role_state.get(InputRole::Pause).1 & 0b1111 == 0b1000 {
                pause = !pause;
            }
            if input_role_state.get(InputRole::OneTick).1 & 0b1111 == 0b1000 {
                one_tick = true;
            }
            bg.1.set_view_pos(view_pos.x, view_pos.y);
        }
        if !renderd {
            for y in 0..WORLD_SIZE.1 {
                for x in 0..WORLD_SIZE.0 {
                    let cell = world.read_cell((x, y));
                    bg.1.set_cur_pos(x as i32, y as i32)
                        .put_code(if cell.state == CellState::Live { '*' } else { ' ' })
                        .put_palette(1)
                    ;
                }
            }
            bg.0.set_cur_pos(1, 2)
                .put_string(&format!("Gen:{} Lives:{}  ", &g_count, &lives), Some(&CharAttributes::new(2, BgSymmetry::Normal)))
            ;
            renderd = true;
        }
        if one_tick || !pause && display_info.f_count % wait == 0 {
            for _ in 0..g_span {
                lives += world.update_world();
            }
            g_count += g_span;
            g_span = 1;
            renderd = false;
            if one_tick {
                one_tick = false;
                pause = true;
            }
        }
        bg.0.set_cur_pos(1, 1)
            .put_string(&format!("({}, {})", view_pos.x, view_pos.y), Some(&CharAttributes::new(3, BgSymmetry::Normal)))
            .put_code_n(' ', 10)
        ;
        if wait_and_update::doing(&mut window, &mut bg, &mut display_info, &keyboard_map, &mut input_role_state) {
             break 'mail_loop;
        }
    }
    sdl_context.mouse().show_cursor(true);
}
