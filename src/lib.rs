#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use std::os::raw::{c_double, c_int};

mod world;
mod engine;
use world::{WorldState};

lazy_static! {
    static ref WORLD_STATE: Mutex<WorldState> = Mutex::new(WorldState::new());
}

// imported js functions
extern "C" {
    fn js_clear_screen(_: c_int);
    fn js_update();
    fn js_request_tick();
    fn js_start_interval_tick(_: c_int);
    fn js_draw_tile(_: c_double, _: c_double, _: c_double, _: c_int, _: c_int, _: c_int, _: c_int);
}

#[no_mangle]
pub extern "C" fn init(debug: i32, render_interval_ms: i32) {
    let world = &mut WORLD_STATE.lock().unwrap();
    world.debug = if debug == 1 { true } else { false };
    unsafe {
        if world.debug {
            js_start_interval_tick(render_interval_ms);
            return;
        };
        js_request_tick();
    }
}

#[no_mangle]
pub extern "C" fn tick() {
    clear_screens();
    update();
    draw();
    unsafe {
        js_request_tick();
    }
}

pub fn clear_screens() {
    unsafe {
        js_clear_screen(0);
    }
}

pub fn update() {
    unsafe {
        js_update();
    }
}

pub fn draw() {
    let world = &mut WORLD_STATE.lock().unwrap();
    let mut start_target = world.get_tile_at(0, 0);
    start_target.color.h = 220;
    let end_target = world.get_tile_at(8, 12);
    unsafe {
        for t in world.tiles.iter() {
            js_draw_tile(
                t.transform.pos_x,
                t.transform.pos_y,
                t.transform.scale_x,
                t.color.h as i32,
                t.color.s as i32,
                t.color.l as i32,
                t.color.a as i32,
            );
        }
        // draw the start tile
        js_draw_tile(
            start_target.transform.pos_x,
            start_target.transform.pos_y,
            start_target.transform.scale_x,
            220,
            start_target.color.s as i32,
            start_target.color.l as i32,
            start_target.color.a as i32,
        );
        // draw the end tile
        js_draw_tile(
            end_target.transform.pos_x,
            end_target.transform.pos_y,
            end_target.transform.scale_x,
            280,
            end_target.color.s as i32,
            end_target.color.l as i32,
            end_target.color.a as i32,
        );
    }
}
