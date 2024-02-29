pub mod renderer;
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

const POLL_SLEEP_TIME: time::Duration = time::Duration::from_millis(100);
const WAIT_TIME: time::Duration = time::Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Wait,
    WaitUntil,
    Poll,
}


struct VoronoiCell
{
    pos: (f32,f32),
    hue: f32
}

impl VoronoiCell
{
    fn random() -> Self
    {
	Self{
	    pos: (rand::random::<f32>(), rand::random::<f32>()),
	    hue: rand::random::<f32>()
	}
    }
    fn hash_grid(&self, radius: f32) -> i32
    {
	let (x,y) = self.pos;
	let i: i32 = (x/radius).floor() as i32;
	let j: i32 = (y/radius).floor() as i32;
	let hash: i32 = ((i+j).pow(2)+3*i+j)/2;
	return hash
    }
}



unsafe fn set_uniform(gl: &glow::Context, program: NativeProgram, name: &str, value: f32) {
    let uniform_location = gl.get_uniform_location(program, name);
    // See also `uniform_n_i32`, `uniform_n_u32`, `uniform_matrix_4_f32_slice` etc.
    gl.uniform_1_f32(uniform_location.as_ref(), value)
}



fn main() {
    let centroids = (0..100).map(|i| VoronoiCell::random());
    unsafe {
	let (rdr, event_loop) = Renderer::new("oui oui baguette", (1600./2.0, 900./2.0));

	let vertices = vec![
            Vertice{pos: [-0.5, -0.5], label: [0]},
            Vertice{pos: [0.0, 0.5], label: [1]},
            Vertice{pos: [0.5, -0.5], label: [2]},
            Vertice{pos: [0.0, -1.0], label: [0]},
	];
	let indices = vec![0,1,2, 0,2,3];
	let (vbo, vao, ibo) = rdr.create_mesh(vertices.as_slice(), indices.as_slice());
	
	
	println!("SIZE OF VERTICE {}",  std::mem::size_of::<Vertice>() as i32);
        let program = rdr.gl.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#"
	    #version 310 es
            in vec2 pos;
            in int label;
            out vec3 vert;
            const vec3 palette[3] = vec3[](vec3(1,0,0),vec3(0,1,0),vec3(0,0,1));
            void main()
            {
                gl_Position = vec4(pos, 0.0, 1.0);
                vert =palette[label];
            }"#,
            r#"
	    #version 310 es
            precision mediump float;
            in vec3 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 1.0);
            }"#,
        );

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
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

        // We handle events differently between targets
	println!("MArcooooo");
        {
            use glutin::prelude::GlSurface;
            use winit::event::{Event, WindowEvent};

	    let mut mode = Mode::Wait;
	    let mut request_redraw = false;
	    let mut wait_cancelled = false;
	    let mut close_requested = false;
	    
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
				rdr.gl.clear(glow::COLOR_BUFFER_BIT);
				rdr.draw_mesh(vao);
//				rdr.gl.draw_arrays(glow::TRIANGLES, 0, 3);
				rdr.gl_surface.swap_buffers(&rdr.gl_context).unwrap();
                            }
                            _ => (),
			}
                    },
		    // no fucking clue of if that's how I'm supposed to manage things
		    // I'll check about that when I'll get some free time (lfmao)
		    Event::AboutToWait => {
			if request_redraw && !wait_cancelled && !close_requested {
//			    window.request_redraw();
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
				thread::sleep(POLL_SLEEP_TIME);
				//                            event_loop.set_control_flow(ControlFlow::Poll);
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
