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



fn main()
{
    let mut graph = Graph::new();
    let centroids = (0..100000)
	.for_each(|i|
		  {
		      let cell = match rand::random::<u32>() % 3
		      {
		      	  0 => Cell::Air,
		      	  1 => Cell::Dirt,
		      	  _ => Cell::Stone
		      };
		      
		      let radius = 1.0;
		      let x = (rand::random::<f32>()*2.0 - 1.0)*radius;
		      let y = (rand::random::<f32>()*2.0 - 1.0)*radius;
		      graph.add_cell(cell, glam::Vec2{x,y});
		  }
	);
    graph.recompute_all();
    let vertices: Vec<Vertice> = graph.nodes.iter().map(|Node{cell, pos, ..}| Vertice{pos: [pos.x, pos.y], label: [rand::random::<i32>() % 16]}).collect();
    unsafe {
	let (rdr, event_loop) = Renderer::new("oui oui baguette", (1600./2.0, 900./2.0));

	// let vertices = vec![
        //     Vertice{pos: [-0.5, -0.5], label: [0]},
        //     Vertice{pos: [0.0, 0.5], label: [1]},
        //     Vertice{pos: [0.5, -0.5], label: [2]},
        //     Vertice{pos: [0.0, -1.0], label: [0]},
	// ];
	//	let indices = vec![0,1,2, 0,2,3];
	let indices: Vec<u32> = graph.triangles.iter().cloned().map(|(i,j,k)| [i as u32,j as u32,k as u32]).flatten().collect();
	let gl_obj = rdr.create_mesh(vertices.as_slice(), indices.as_slice());
	
	
	println!("SIZE OF VERTICE {}",  std::mem::size_of::<Vertice>() as i32);
        let program = rdr.gl.create_program().expect("Cannot create program");

        let (vertex_shader_source, geometry_shader_source, fragment_shader_source) = (
            r#"
	    #version 330 core
            in vec2 pos;
            in int label;
            out int vlabel;            
            void main()
            {
                vlabel = label;
                gl_Position = vec4(pos, 0, 1);
            }"#,

	    r#"
#version 330 core
layout (triangles) in;
layout(triangle_strip, max_vertices=18) out;
in int vlabel[];
flat out int flabel;
flat out vec2 centroid_pos;
out vec2 fpos;

vec2 circumcenter(vec2 p1, vec2 p2, vec2 p3)
{
    vec2 v12 = p1 - p2;
    vec2 v31 = p3 - p1;
    vec2 v23 = p2 - p3;
    float num = 2.0*pow(determinant(mat2(v12, v23)), 2);
    float u = - dot(v23, v23)*dot(v12, v31);        
    float v = - dot(v31, v31)*dot(v12, v23);        
    float w = - dot(v12, v12)*dot(v31, v23);

    return (p1*u + p2*v + p3*w)/num;
}


void add_triangle(vec4 p0, vec4 p1, vec4 p2, int lab)
{
    flabel = lab;
    centroid_pos = p0.xy/p0.w;
    gl_Position = p0; 
    fpos = p0.xy/p0.w; 
    EmitVertex();

    fpos = p1.xy/p1.w; 
    gl_Position = p1; 
    EmitVertex();

    fpos = p2.xy/p2.w; 
    gl_Position = p2;
    EmitVertex();
    EndPrimitive(); 
}

 
void main()
{
    vec4 pos0 =  gl_in[0].gl_Position;
    vec4 pos1 =  gl_in[1].gl_Position;
    vec4 pos2 =  gl_in[2].gl_Position;
    vec4 pos012 = vec4(circumcenter(pos0.xy/pos0.w, pos1.xy/pos1.w, pos2.xy/pos2.w), 0, 1);
    vec4 pos01 = pos0 + pos1;
    vec4 pos02 = pos0 + pos2;
    vec4 pos12 = pos1 + pos2;

    add_triangle(pos0, pos01, pos012, vlabel[0]);
    add_triangle(pos0, pos012, pos02, vlabel[0]);
    add_triangle(pos1, pos12, pos012, vlabel[1]);
    add_triangle(pos1, pos012, pos01, vlabel[1]);
    add_triangle(pos2, pos02, pos012, vlabel[2]);
    add_triangle(pos2, pos012, pos12, vlabel[2]);


}
"#,

	    
            r#"
	    #version 330 core
            precision mediump float;
            flat in int flabel;
            flat in vec2 centroid_pos;
            in vec2 fpos;
            out vec4 color;
uniform vec2 uScreenSize;
            const vec3 palette[16] =vec3[](
vec3(0, 32.+11., 48.+6.)/255.0,
vec3(0*16+7,3*16+6,4*16+2)/255.,
vec3(5*16+8,6*16+14,7*16+5)/255.0,
vec3(6*16+5,7*16+11,8*16+3)/255.0,
vec3(8*16+3,9*16+4,9*16+6)/255.0,
vec3(9*16+3,10*16+1,10*16+1)/255.0,
vec3(14*16+14,14*16+8,13*16+5)/255.0,
vec3(15*16+13,15*16+6,14*16+3)/255.0,
vec3(11*16+5,8*16+9,0*16+0)/255.0,
vec3(12*16+11,4*16+11,1*16+6)/255.0,
vec3(13*16+12,3*16+2,2*16+15)/255.0,
vec3(13*16+3,3*16+6,8*16+2)/255.0,
vec3(6*16+12,7*16+1,12*16+4)/255.0,
vec3(2*16+6,8*16+11,13*16+2)/255.0,
vec3(2*16+10,10*16+1,9*16+8)/255.0,
vec3(8*16+5,9*16+9,0)/255.0);

            void main()
            {
                float d = distance(centroid_pos, fpos);
                vec2 frag_pos = gl_FragCoord.xy/uScreenSize;
                float attenuation = exp(-d*d*6.0);
                color = vec4(palette[flabel]*attenuation, 1.0);
//                if (d < 0.003) color = vec4(1,1,1,1);


            }"#,
        );

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::GEOMETRY_SHADER, geometry_shader_source),
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

	let loc_screenres = rdr.gl.get_uniform_location(program, "uScreenSize");
	
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
				t+= 0.1;
				let new_clock2 = std::time::Instant::now();
				let w = rdr.gl_surface.width().unwrap();
				let h = rdr.gl_surface.height().unwrap();
				rdr.gl.uniform_2_f32(loc_screenres.as_ref(), w as f32, h as f32);
				rdr.gl.clear(glow::COLOR_BUFFER_BIT);
				rdr.draw_mesh(&gl_obj);
				println!("t={}", t);
				let draw_time = new_clock2.elapsed().as_secs_f32();
				rdr.gl_surface.swap_buffers(&rdr.gl_context).unwrap();
				request_redraw = true;
				let swap_time = new_clock2.elapsed().as_secs_f32();
				let new_clock = std::time::Instant::now();
				let elapsed = new_clock.duration_since(clock);
				clock = new_clock;
				let delta = elapsed.as_secs_f32();
				println!("Tick time: {} secs ({} FPS)  | frame drawing: {} secs or {} with swap",
					 delta, 1.0/delta, draw_time, swap_time);
				

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
