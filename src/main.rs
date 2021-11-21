use std::time::{Instant, Duration};
use rodio::{OutputStream, Sink};
use rodio::source::{SineWave, Source};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit::dpi::PhysicalSize;
use winit_input_helper::WinitInputHelper;
use pixels::{Pixels, SurfaceTexture};
use rand::prelude::*;

use lakhesis::{ Model, ITERATIONS, MAX_DROPS };

// number of PNG frames to create for experimental video function
const VIDEO_FRAME_COUNT: usize = 600;       // 10 seconds at 60fps

fn main() {
    println!("Starting Lakhesis");
    animate_sandpile();
}

pub fn measure_elapsed_time <F: FnOnce()>(f: F) -> Duration {
    let t0 = Instant::now();
    f();
    Instant::now() - t0
}

pub fn animate_sandpile() {
    let mut rng = rand::thread_rng();
    let mut input = WinitInputHelper::new();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize { width: 1280, height: 720 })
        .with_title("| L A K H E S I S | [Q]uit | [N]ew | [P]ause/resume | [Spacebar] step |
            [Up] faster | [Down] slower | [S]napshot | [A]dd | [C]olors | [R]andom |")
        .build(&event_loop)
        .unwrap();
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    println!("Window Inner Size: {:?}", &window_size);
    let mut model = Model::default();
    let mut pixels = Pixels::new(model.table.width as u32, model.table.height as u32, surface_texture)
        .expect("Unable to create new surface texture");
    let mut paused = false;
    let mut reset: bool = false;
    let mut audio: bool = false;
    let mut video: usize = 0;
    let mut additional_cells: usize = rng.gen_range(0..MAX_DROPS);
    println!("Random additional sandpiles: {}\nTimes {:?}\nLocations {:?}",
        &additional_cells, &model.drop_times, &model.drop_cells);
    let mut ac: usize = 0;
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sink: Vec<Sink> = Vec::new();
    for _i in 0..MAX_DROPS {
        sink.push(Sink::try_new(&stream_handle).unwrap());
    }
    let mut max_avalanche: usize = 0;
    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            // escape Q key to quit
            if input.key_released(VirtualKeyCode::Q) || input.quit() {
                *control_flow = ControlFlow::Exit;
                println!("Exiting Lakhesis");
                return;
            }
            // increase interval by 10x up to 100_000x
            if input.key_pressed(VirtualKeyCode::Up) {
                if model.interval < 100_000 && video == 0 { model.interval *= 10; }
                println!("interval = {}", &model.interval);
            }
            // decrease interval by 10x down to 1
            if input.key_pressed(VirtualKeyCode::Down) {
                if model.interval > 1 && video == 0 { model.interval /= 10; }
                println!("interval = {}", &model.interval);
            }
            // spacebar to step one frame at a time
            if input.key_pressed(VirtualKeyCode::Space) {
                // spacebar is frame-step, so ensure we're paused
                paused = true;
                window.request_redraw();
            }
            // add a new drop cell manually
            if input.key_pressed(VirtualKeyCode::A) {
                if model.active_cells < 12 {
                    // user takes control of number of drop cells
                    model.random = false;
                    model.active_cells += 1;
                    model.drop_times[model.active_cells - 1] = model.table.total_grains;
                }
                println!("added drop cell - current active cells = {}", &model.active_cells);
            }
            // cause a random color change for sandpiles
            if input.key_pressed(VirtualKeyCode::C) {
                // set random variable to false so color choice persists
                model.random = false;
                model.random_colors();
                println!("color change - random is now set to {:?}", &model.random);
            }
            //turn audio on/off
            if input.key_pressed(VirtualKeyCode::L) {
                audio = !audio;
            }
            // new simulation - reset to default
            if input.key_pressed(VirtualKeyCode::N) {
                reset = true;
            }
            // pause/restart the simulation
            if input.key_pressed(VirtualKeyCode::P) {
                paused = !paused;
            }
            // toggle random colors and drop cells
            if input.key_pressed(VirtualKeyCode::R) {
                model.random = !model.random;
                println!("random is now set to {:?}", &model.random);
            }
            // save image and model up to this point
            if input.key_released(VirtualKeyCode::S) {
                model.paint();
                println!("Lakhesis_{:07}.png exported", &model.table.total_grains);
            }
            // collect frames at set interval for use as a GIF or video
            if input.key_pressed(VirtualKeyCode::V) {
                video = VIDEO_FRAME_COUNT;
                println!("video started at interval = {}", &model.interval);
            }
            // resize the window
            // table size kept at 1280x720 and window is not recentered
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }
        }
        // if !paused add a sand grain and resolve unstable sandpiles
        if !paused || input.key_pressed(VirtualKeyCode::Space) {
            let dt = measure_elapsed_time(|| {
                model.add_grain(ac);
                // check if a new drop cell is pending if random = true
                if model.random {
                    for i in 1..additional_cells + 1 {
                        if model.drop_times[i] == model.table.total_grains {
                            model.active_cells += 1;
                            model.random_colors();
                        }
                    }
                }
                if model.active_cells - 1 > ac { ac += 1; } else { ac = 0; }
            });
            if audio {
                if model.avalanche > max_avalanche { max_avalanche = model.avalanche.clone() };
                let source1 = SineWave::new((1000.0 - ((model.avalanche as f32 / max_avalanche as f32) * 800.0)).trunc() as u32)
                    .take_duration(Duration::from_secs_f32(dt.as_secs() as f32 + dt.subsec_nanos() as f32 * 1e-9)
                    ).amplify(0.10);
                let source2 = SineWave::new((2000.0 - ((model.avalanche as f32 / max_avalanche as f32) * 1600.0)).trunc() as u32)
                    .take_duration(Duration::from_secs_f32(dt.as_secs() as f32 + dt.subsec_nanos() as f32 * 1e-9)
                    ).amplify(0.08);
                sink[ac].append(Source::mix(source1, source2));
            }
        }
        if model.table.total_grains % model.interval == 0 {
            window.request_redraw();
            if video > 0 {
                model.paint();
                video -= 1;
                if video == 0 { println!("video recorded {} frames", &VIDEO_FRAME_COUNT); }
            }
        }
        if model.table.total_grains >= ITERATIONS || reset {
            model = Model::default();
            paused = false;
            reset = false;
            additional_cells = rng.gen_range(0..MAX_DROPS);
            println!("new simulation started\nRandom additional sandpiles: {}\nTimes {:?}\nLocations {:?}",
                &additional_cells, &model.drop_times, &model.drop_cells);
            ac = 0;
            window.request_redraw();
        }
        if let Event::RedrawRequested(_) = event {
            model.draw(pixels.get_frame());
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}
