#![feature(iter_array_chunks)]

pub mod renderer;
pub mod voronoi;
use voronoi::*;
use renderer::{Renderer, Vertice};
use glow::*;

use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

use std::thread;

#[cfg(not(web_platform))]
use std::time;
#[cfg(web_platform)]
use web_time as time;

const POLL_SLEEP_TIME: time::Duration = time::Duration::from_millis(10);
const WAIT_TIME: time::Duration = time::Duration::from_millis(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Wait,
    WaitUntil,
    Poll,
}





unsafe fn set_uniform(gl: &glow::Context, program: NativeProgram, name: &str, value: f32) {
    let uniform_location = gl.get_uniform_location(program, name);
    // See also `uniform_n_i32`, `uniform_n_u32`, `uniform_matrix_4_f32_slice` etc.
    gl.uniform_1_f32(uniform_location.as_ref(), value)
}


use bluenoise::WrappingBlueNoise;
use rand::{SeedableRng, Rng};
use rand_pcg::Pcg64Mcg;


use std::io::Read;

// /**
// Expect the path of a folder containing a file "vertex.glsl" and "fragment.glsl" as well as an optional "geometry.glsl" and returns their content.
//  */
// macro_rules! open_shaders
// {
//     ($path:expr) =>
//     {
// 	{
// 	    let folder_path = std::path::Path::new($path);
// 	    println!("Opening shaders at {:?}", folder_path);
// 	    let vs_path = folder_path.join("vertex.glsl");
// 	    let gs_path = folder_path.join("geometry.glsl");
// 	    let fs_path = folder_path.join("fragment.glsl");
	    
// 	    let vs = include!(vs_path.as_os_str());
// 	    let gs = include!(vs_path);
// 	    let fs = include!(vs_path);
// 	    Ok([vs,gs,fs])
// 	}	
//     }
// }

fn main()
{
    let mut graph = Voronoi::new();
    let mut rng =  Pcg64Mcg::seed_from_u64(10);

    let mut noise = WrappingBlueNoise::from_rng(1.0, 1.0, 0.01, Pcg64Mcg::seed_from_u64(100));
    let noise = noise.with_samples(10);
    
    let mut max_radius = 0.0;
    let centroids = noise.take(1000)
	.for_each(|pos|
		  {
		      let cell = match rand::random::<u32>() % 3
		      {
		      	  0 => Cell::Air,
		      	  1 => Cell::Dirt,
		      	  _ => Cell::Stone
		      };
		      let p = pos*2.0 - glam::Vec2::new(1.0,1.0);
		      graph.add_cell(cell, p);
		      let rad = p.length_squared();
		      if rad > max_radius {max_radius = rad};
		  }
	);
    println!("Max radius: {}", max_radius);
    graph.recompute_all();
    let mut vertices: Vec<Vertice> = vec![];
    let mut indices: Vec<u32> = vec![];
    for (i,_cell) in graph.nodes_values.iter().enumerate()
    {
	let label = [(rng.gen::<u32>() % 16) as i32];
	let poses = graph.nodes_corner[i].iter().map(|corner_id| graph.corners_pos[*corner_id]);
	let n = poses.len() as u32;
	let start = vertices.len() as u32;
	for pos in poses
	{
	    vertices.push(Vertice{pos: [pos.x,pos.y], label});
	}
	for i in 2..n
	{
	    indices.push(start);
	    indices.push(start+i-1);
	    indices.push(start+i);
	}
    }
//    println!("VERTICES: {:?}", &vertices);
 //   println!("INDICES : {:?}", &indices);
 //graph.nodes_pos.iter().map(|Node{cell, pos, ..}| Vertice{pos: [pos.x, pos.y], label: [(rng.gen::<u32>() % 16) as i32]}).collect();
    unsafe {
	let (rdr, event_loop) = Renderer::new("oui oui baguette", (1600./2.0, 900./2.0), true);

	// let vertices = vec![
        //     Vertice{pos: [-0.5, -0.5], label: [0]},
        //     Vertice{pos: [0.0, 0.5], label: [1]},
        //     Vertice{pos: [0.5, -0.5], label: [2]},
        //     Vertice{pos: [0.0, -1.0], label: [0]},
	// ];
	//	let indices = vec![0,1,2, 0,2,3];
//	let indices: Vec<u32> =vec![];// graph.triangles.iter().cloned().map(|(i,j,k)| [i as u32,j as u32,k as u32]).flatten().collect();
	let gl_obj = rdr.create_mesh(vertices.as_slice(), indices.as_slice());
//	let gl_obj = rdr.create_mesh(&[Vertice{pos:[0.,-0.5], label:[0]}, Vertice{pos:[1.,0.], label:[1]}, Vertice{pos:[0.,0.5], label:[0]}], &[0,1,2]);	
	
	println!("SIZE OF VERTICE {}",  std::mem::size_of::<Vertice>() as i32);
        let program = rdr.gl.create_program().expect("Cannot create program");



        let shader_sources = [
            (glow::VERTEX_SHADER, howto::shaders::simple::VERTEX),
//            (glow::GEOMETRY_SHADER, howto::shaders::old_voronoi::GEOMETRY),
            (glow::FRAGMENT_SHADER, howto::shaders::simple::FRAGMENT),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = rdr.gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            rdr.gl.shader_source(shader, shader_source);
            rdr.gl.compile_shader(shader);
            if !rdr.gl.get_shader_compile_status(shader) {
                panic!("{}", rdr.gl.get_shader_info_log(shader));
            }
            rdr.gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        rdr.gl.link_program(program);
        if !rdr.gl.get_program_link_status(program) {
            panic!("{}", rdr.gl.get_program_info_log(program));
        }

        for shader in shaders {
            rdr.gl.detach_shader(program, shader);
            rdr.gl.delete_shader(shader);
        }

        rdr.gl.use_program(Some(program));
        rdr.gl.clear_color(0.1, 0.2, 0.3, 1.0);

	let loc_screenres = rdr.gl.get_uniform_location(program, "uScreenSize");
	let loc_projmatrix = rdr.gl.get_uniform_location(program, "uProjMatrix");
	
        // We handle events differently between targets
	println!("MArcooooo");
        {
            use glutin::prelude::GlSurface;
            use winit::event::{Event, WindowEvent};

	    let mut mode = Mode::Poll;
	    let mut request_redraw = false;
	    let mut wait_cancelled = false;
	    let mut close_requested = false;

	    let mut t = 0f32;
	    let mut clock = std::time::Instant::now();
            let _ = event_loop.run(move |event, elwt| {
                match event 
		{
		    Event::WindowEvent { event, .. } => {
			match event {
			    WindowEvent::KeyboardInput {
				event:
				KeyEvent {
				    logical_key: key,
				    state: ElementState::Pressed,
				    ..
				},
				..
			    } => match key.as_ref() {
				 // WARNING: Consider using `key_without_modifiers()` if available on your platform.
				 // See the `key_binding` example
				 Key::Character("1") => {
				 	mode = Mode::Wait;
				 	println!("\nmode: {mode:?}\n");
				 }
				 Key::Character("2") => {
				 	mode = Mode::WaitUntil;
				 	println!("\nmode: {mode:?}\n");
				 }
				 Key::Character("3") => {
				 	mode = Mode::Poll;
				 	println!("\nmode: {mode:?}\n");
				 }
				 Key::Character("r") => {
				 	request_redraw = !request_redraw;
				 	println!("\nrequest_redraw: {request_redraw}\n");
				 }
				Key::Named(NamedKey::Escape) => {
				    close_requested = true;
				}
				_ => (),
			    },
                            WindowEvent::CloseRequested => {
				close_requested = true;
                            }
                            WindowEvent::RedrawRequested => {
				t+= 0.004;
				let new_clock2 = std::time::Instant::now();
				let w = rdr.gl_surface.width().unwrap();
				let h = rdr.gl_surface.height().unwrap();
				rdr.gl.viewport(0,0,w as i32,h as i32);
				// let proj_matrix = glam::Mat4::perspective_lh(
				//     1.0,
				//     w as f32/h as f32,
				//     0.,
				//     100.0
				// );
				let max_coord = w.max(h) as f32;
				let rx = w as f32 / max_coord/2.0;
				let ry = h as f32 / max_coord/2.0;
				let matrix_proj = glam::Mat4::from_diagonal(glam::Vec4::new(1./rx, 1./ry, 1.0, 1.0));
				let matrix_proj = glam::Mat4::perspective_infinite_lh(1.2, w as f32 / h as f32, 0.0);
				let camera_pos = glam::Vec3::new(0.,0., -(t/10.).cos());
				let matrix_view = glam::Mat4::from_translation(glam::Vec3::new(camera_pos.x, camera_pos.y, camera_pos.z));
				let matrix_VP = matrix_proj * matrix_view;
				rdr.gl.uniform_2_f32(loc_screenres.as_ref(), w as f32, h as f32);
				
				rdr.gl.uniform_matrix_4_f32_slice(loc_projmatrix.as_ref(), false, &matrix_VP.to_cols_array());
				rdr.gl.clear(glow::COLOR_BUFFER_BIT);
				rdr.gl.clear_color(t.cos()*(2.*t).cos(), t.sin()*(2.*t).cos(),(2.*t).sin(), 1.);
				rdr.draw_mesh(&gl_obj);
				//println!("t={}", t);
				let draw_time = new_clock2.elapsed().as_secs_f32();
				rdr.gl_surface.swap_buffers(&rdr.gl_context).unwrap();
				request_redraw = true;
				let swap_time = new_clock2.elapsed().as_secs_f32();
				let new_clock = std::time::Instant::now();
				let elapsed = new_clock.duration_since(clock);
				clock = new_clock;
				let delta = elapsed.as_secs_f32();
			//	println!("Tick time: {} secs ({} FPS)  | frame drawing: {} secs or {} with swap",
			//	 	 delta, 1.0/delta, draw_time, swap_time);
				// println!("vertices: {} bytes", std::mem::size_of::<Vertice>()*vertices.len());
				

                            }
                            _ => (),
			}
                    },
		    // no fucking clue of if that's how I'm supposed to manage things
		    // I'll check about that when I'll get some free time (lfmao)
		    Event::AboutToWait => {
			if request_redraw && !wait_cancelled && !close_requested {
			    rdr.window.request_redraw();
			}

			match mode {
			    Mode::Wait => elwt.set_control_flow(ControlFlow::Wait),
			    Mode::WaitUntil => {
				if !wait_cancelled {
			            elwt.set_control_flow(ControlFlow::WaitUntil(
					time::Instant::now() + WAIT_TIME,
			            ));
				}
			    }
			    Mode::Poll =>
			    {
//				thread::sleep(POLL_SLEEP_TIME);
			//	event_loop.set_control_flow(ControlFlow::Poll);
			    }
			};

			if close_requested {
			    elwt.exit();
			}
		    },
		    _ => ()  
		}
            });
        }


    }
}
