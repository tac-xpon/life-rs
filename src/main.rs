use std::collections::BTreeMap;

mod life_cell;
use life_cell::*;

mod direction;
use direction::*;

mod input_role;
use input_role::*;

mod bgchar_data;
mod bgpal_data;

mod game_window;
use game_window::*;

mod wait_and_update;

use bgsp_lib2::{
    bgsp_common::*,
    bg_plane::*,
};

const WORLD_SIZE: (usize, usize) = (512, 512);

const FULL_SCREEN: bool = false;
const VM_RECT_SIZE: (i32, i32) = (120 * PATTERN_SIZE as i32, 120 * PATTERN_SIZE as i32);
const ROTATION: Direction = Direction::Normal;
const PIXEL_SCALE: i32 = 1;
const WINDOW_MARGIN: i32 = 0;
const BG0_RECT_SIZE: (i32, i32) = (128, 120);
const BG1_RECT_SIZE: (i32, i32) = (WORLD_SIZE.0 as i32, WORLD_SIZE.1 as i32);

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut game_window = GameWindow::new(
        video_subsystem,
        FULL_SCREEN,
        VM_RECT_SIZE,
        ROTATION,
        PIXEL_SCALE,
        WINDOW_MARGIN,
    );

    let mut keyboard_map: BTreeMap<piston_window::Key, Vec<_>> = BTreeMap::new();
    {
        let key_set_list = [
            (piston_window::Key::D1,    InputRole::Progress1),
            (piston_window::Key::D2,    InputRole::Progress2),
            (piston_window::Key::D3,    InputRole::Progress4),
            (piston_window::Key::D4,    InputRole::Progress8),
            (piston_window::Key::P,     InputRole::Pause),
            (piston_window::Key::O,     InputRole::OneTick),
            (piston_window::Key::H,     InputRole::Home),
            (piston_window::Key::Z,     InputRole::Button0),
            (piston_window::Key::Space, InputRole::Button0),
            (piston_window::Key::W,     InputRole::Up),
            (piston_window::Key::D,     InputRole::Right),
            (piston_window::Key::S,     InputRole::Down),
            (piston_window::Key::A,     InputRole::Left),
            (piston_window::Key::Up,    InputRole::Up),
            (piston_window::Key::Right, InputRole::Right),
            (piston_window::Key::Down,  InputRole::Down),
            (piston_window::Key::Left,  InputRole::Left),
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

    let mut bg_texture_bank = BgTextureBank::new(
        &bgchar_data::BG_PATTERN_TBL,
        &bgpal_data::COLOR_TBL,
        game_window.pixel_scale() as i32,
    );
    let rc_bg_texture_bank = Rc::new(RefCell::new(&mut bg_texture_bank));
    let mut bg = {
        let mut bg0 = BgPlane::new(
            BG0_RECT_SIZE,
            VM_RECT_SIZE,
            rc_bg_texture_bank.clone(),
        );
        bg0.set_base_symmetry(BgSymmetry::Normal);

        let mut bg1 = BgPlane::new(
            BG1_RECT_SIZE,
            VM_RECT_SIZE,
            rc_bg_texture_bank.clone(),
        );
        bg1.set_base_symmetry(BgSymmetry::Normal);
        (bg0, bg1)
    };

    if game_window.full_screen() {
        sdl_context.mouse().show_cursor(false);
    }

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
        if one_tick || !pause && game_window.f_count() % wait == 0 {
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
        if wait_and_update::doing(&mut game_window, &mut bg, &keyboard_map, &mut input_role_state) {
             break 'mail_loop;
        }
    }
    sdl_context.mouse().show_cursor(true);
}
