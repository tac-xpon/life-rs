use crate::{
    direction::*,
    // input_role::*,
    GameWindow,
    DisplayInfo,
};
// use std::collections::BTreeMap;
use piston_window::*;
use bgsp_lib2::{
    bg_plane::*,
    // sp_resources::*
};

pub fn doing(
    window: &mut GameWindow,
    // spr: &mut SpResources,
    bg: &mut (BgPlane, BgPlane),
    info: &mut DisplayInfo,
    // keyboard_map: &BTreeMap<piston_window::Key, Vec<InputRole>>,
    // input_role_state: &mut InputRoleState,
) -> bool {
    let mut texture_context = window.create_texture_context();
    let texture_settings = TextureSettings::new();
    let _ = bg.0.rendering();
    let bg0_whole = Texture::from_image(
        &mut texture_context,
        bg.0.whole_image(),
        &texture_settings,
    ).unwrap();
    let _ = bg.1.rendering();
    let bg1_whole = Texture::from_image(
        &mut texture_context,
        bg.1.whole_image(),
        &texture_settings,
    ).unwrap();
    // Sprites
    /*
    let sp_drawn = Texture::from_image(
        &mut texture_context,
        &spr.rendering(info.vm_rect_size.0 as i32, info.vm_rect_size.1 as i32),
        &texture_settings,
    ).unwrap();
    */

    while let Some(event) = window.next() {
    /*
        if let Some(Button::Keyboard(k)) = event.press_args() {
            if let Some(role_list) = keyboard_map.get(&k) {
                for role in role_list { input_role_state.set_true(*role) }
            }
        }
        if let Some(Button::Keyboard(k)) = event.release_args() {
            if let Some(role_list) = keyboard_map.get(&k) {
                for role in role_list { input_role_state.set_false(*role) }
            }
        }
    */
        if let Event::Loop(Loop::Render(_)) = event {
            // input_role_state.update_history();
            let window_size = window.size();
            window.draw_2d(&event, |context, graphics, _device| {
                let (vm_rect_width, vm_rect_height, pixel_scale, margin_2x) = (
                    info.vm_rect_size.0,
                    info.vm_rect_size.1,
                    info.pixel_scale,
                    info.margin * 2,
                );
                let (zoom, h_offset, v_offset) = {
                    let view_rect = {
                        let (width, height) = (vm_rect_width * pixel_scale, vm_rect_height * pixel_scale);
                        match info.rotation {
                            Direction::Up    | Direction::Down => (width, height),
                            Direction::Right | Direction::Left => (height, width),
                        }
                    };
                    let h_zoom = window_size.width / ((view_rect.0 + margin_2x) as f64);
                    let v_zoom = window_size.height / ((view_rect.1  + margin_2x) as f64);
                    let zoom = h_zoom.min(v_zoom);
                    let h_offset =  (window_size.width - (view_rect.0 as f64) * zoom) / 2.0;
                    let v_offset =  (window_size.height - (view_rect.1 as f64) * zoom) / 2.0;
                    (zoom, h_offset, v_offset)
                };
                let base_transform = context.transform.zoom(zoom).trans(h_offset / zoom, v_offset / zoom);
                let transform = match info.rotation {
                    Direction::Up    => base_transform.rot_deg(  0.0).trans(0.0, 0.0),
                    Direction::Right => base_transform.rot_deg( 90.0).trans(0.0, -((vm_rect_height * pixel_scale) as f64)),
                    Direction::Down  => base_transform.rot_deg(180.0).trans(-((vm_rect_width * pixel_scale) as f64), -((vm_rect_height * pixel_scale) as f64)),
                    Direction::Left  => base_transform.rot_deg(270.0).trans(-((vm_rect_width * pixel_scale) as f64), 0.0),
                };
                let draw_inside = draw_state::DrawState::new_inside();
                if info.f_count < 4 {
                    // Initialize
                    graphics.clear_color([0.0, 0.0, 0.0, 1.0]);
                    graphics.clear_stencil(0);
                    rectangle::Rectangle::new([1.0; 4]).draw(
                        [0.0, 0.0, (vm_rect_width * pixel_scale) as f64, (vm_rect_height * pixel_scale) as f64],
                        &draw_state::DrawState::new_clip(),
                        transform,
                        graphics,
                    );
                } else {
                    // Clear
                    rectangle::Rectangle::new([0.0, 0.0, 0.0, 1.0]).draw(
                        [0.0, 0.0, (vm_rect_width * pixel_scale) as f64, (vm_rect_height * pixel_scale) as f64],
                        &draw_inside,
                        transform,
                        graphics,
                    );
                }
                // BG1
                image::draw_many(
                    bg.1.draw_rects(), [1.0, 1.0, 1.0, 1.0],
                    &bg1_whole,
                    &draw_inside,
                    transform,
                    graphics,
                );
                // Sprites
                /*
                image::Image::new().draw(
                    &sp_drawn,
                    &draw_inside,
                    transform,
                    graphics,
                );
                */
                // BG0
                image::draw_many(
                    bg.0.draw_rects(), [1.0, 1.0, 1.0, 1.0],
                    &bg0_whole,
                    &draw_inside,
                    transform,
                    graphics,
                );
            });
            info.f_count += 1;
            return false;
        }
    }
    true
}
