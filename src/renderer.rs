use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, SwapInterval},
};

use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use std::num::NonZeroU32;

use glutin::context::PossiblyCurrentContext;
use glutin::surface::{WindowSurface, Surface};
use glow::{Context, HasContext};

use bytemuck::{Pod, Zeroable};



pub struct Renderer
{
    pub gl: Context,
    pub gl_surface:Surface<WindowSurface>,
    pub gl_context: PossiblyCurrentContext,
    pub window: winit::window::Window,
}




/** whatever implements this is a vertice */
pub trait ThatAVertice
{
    unsafe fn setup_vao(gl: &glow::Context);
}

/** this macro is used to create a new vertex structure, and implements automaticaly everything needed to use it on the gpu */
macro_rules! create_vertex
{
    ($name:ident, $($param:ident, $param_type: ty, $param_size: expr),+) =>
    {
	#[derive(Debug, Clone, Copy, Pod, Zeroable)]
	#[repr(C)]
	pub struct $name
	{
	    $(pub $param: [$param_type; $param_size]),+
	}
	impl ThatAVertice for $name
	{
	    unsafe fn setup_vao(gl: &glow::Context)
	    {
		let layout: [usize;  0 $(+ 1 + $param_size*0)+] = [$($param_size* std::mem::size_of::<$param_type>(),)+];
		let total_size: usize =  std::mem::size_of::<Self>();
		let mut i: i32 = -1;
		let mut offset = 0;
		$(
		    i+=1; // we start at -1 to avoid a warning
		    gl.enable_vertex_attrib_array(i as u32);
		    // there are other methods such as vertex_attrib_pointer_i32/f64, but they are unecessary in our case
		    gl.vertex_attrib_pointer_f32(i as u32, $param_size, glow::FLOAT, false, total_size as i32, offset);
		    offset += layout[i as usize] as i32;
		)+
	    	
	    }
	}
    }
}

create_vertex!(Vertice, pos, f32, 2, label, i32, 1);
pub struct GLObject
{
    vao: glow::NativeVertexArray,
//    vbo: glow::NativeVertexArray,
//    ibo: glow::NativeVertexArray,
    offset: i32,
    nb_indices: i32
}


impl Renderer
{
    pub unsafe fn new(title: &str, base_res: (f32, f32), uncap_fps: bool) -> Result<(Self, winit::event_loop::EventLoop<()>), howto::error::Error>
    {
	let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
        let window_builder = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(base_res.0, base_res.1));

        let template = ConfigTemplateBuilder::new();

        let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

        let (window, gl_config) = display_builder
            .build(&event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

        let gl_display = gl_config.display();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(None))
            .build(raw_window_handle);

        let not_current_gl_context = gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap();

        let window = window.unwrap();

        let attrs = window.build_surface_attributes(Default::default());
        let gl_surface = gl_display
            .create_window_surface(&gl_config, &attrs)
            .unwrap();

        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();
        let gl = glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));

        gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
            .unwrap();
	if uncap_fps
	{
	    gl_surface.set_swap_interval(&gl_context, glutin::surface::SwapInterval::DontWait)?;
	}


	Ok((Self
	 {
	     gl,
	     gl_surface,
	     gl_context,
		window
	 },
	 event_loop))

    }
    
    pub unsafe fn create_mesh<Vertice: ThatAVertice + Pod>
	(&self,
	 vertices: &[Vertice],
	 indices: &[u32]) -> GLObject
    {
	let triangle_vertices_u8 = bytemuck::cast_slice(vertices);
	let triangle_indices_u8 = bytemuck::cast_slice(indices);

	// creating the various buffers
	let vao = self.gl.create_vertex_array().unwrap();
	let vbo = self.gl.create_buffer().unwrap();
	let ibo = self.gl.create_buffer().unwrap();
	// binding them (vao before ibo, very important)
	self.gl.bind_vertex_array(Some(vao));
	self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
	self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ibo));
	// filling them :flushed:
	self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, triangle_vertices_u8, glow::STATIC_DRAW);
	self.gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, triangle_indices_u8, glow::STATIC_DRAW);

	Vertice::setup_vao(&self.gl);

	GLObject
	{
	    vao,
	    offset: 0,
	    nb_indices: indices.len() as i32
	}
    }

    pub unsafe fn draw_mesh(&self, obj: &GLObject)
    {
	self.gl.bind_vertex_array(Some(obj.vao));
	self.gl.draw_elements(glow::TRIANGLES, obj.nb_indices, glow::UNSIGNED_INT, obj.offset);
	self.gl.bind_vertex_array(None);

    }
    
}
