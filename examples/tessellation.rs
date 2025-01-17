#[macro_use]
extern crate glium;

use glium::index::PrimitiveType;
use glium::{Display, Surface};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

mod support;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

struct Application {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>,
    pub program: glium::Program,
    pub tess_level: i32,
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium tesselation example";

    fn new(display: &Display<WindowSurface>) -> Self {
        // building the vertex buffer, which contains all the vertices that we will draw
        let vertex_buffer = {
            glium::VertexBuffer::new(
                display,
                &[
                    Vertex {
                        position: [-0.5, -0.5],
                    },
                    Vertex {
                        position: [0.0, 0.5],
                    },
                    Vertex {
                        position: [0.5, -0.5],
                    },
                ],
            )
            .unwrap()
        };

        // building the index buffer
        let index_buffer = glium::IndexBuffer::new(
            display,
            PrimitiveType::Patches {
                vertices_per_patch: 3,
            },
            &[0u16, 1, 2],
        )
        .unwrap();

        // compiling shaders and linking them together
        let program = glium::Program::new(
            display,
            glium::program::SourceCode {
                vertex_shader: "
                    #version 140

                    in vec2 position;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                    }
                ",
                fragment_shader: "
                    #version 140

                    in vec3 color;
                    out vec4 f_color;

                    void main() {
                        f_color = vec4(color, 1.0);
                    }
                ",
                geometry_shader: Some(
                    "
                    #version 330

                    uniform mat4 matrix;

                    layout(triangles) in;
                    layout(triangle_strip, max_vertices=3) out;

                    out vec3 color;

                    float rand(vec2 co) {
                        return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
                    }

                    void main() {
                        vec3 all_color = vec3(
                            rand(gl_in[0].gl_Position.xy + gl_in[1].gl_Position.yz),
                            rand(gl_in[1].gl_Position.yx + gl_in[2].gl_Position.zx),
                            rand(gl_in[0].gl_Position.xz + gl_in[2].gl_Position.zy)
                        );

                        gl_Position = matrix * gl_in[0].gl_Position;
                        color = all_color;
                        EmitVertex();

                        gl_Position = matrix * gl_in[1].gl_Position;
                        color = all_color;
                        EmitVertex();

                        gl_Position = matrix * gl_in[2].gl_Position;
                        color = all_color;
                        EmitVertex();
                    }
                ",
                ),
                tessellation_control_shader: Some(
                    "
                    #version 400

                    layout(vertices = 3) out;

                    uniform int tess_level = 5;

                    void main() {
                        gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;

                        gl_TessLevelOuter[0] = tess_level;
                        gl_TessLevelOuter[1] = tess_level;
                        gl_TessLevelOuter[2] = tess_level;
                        gl_TessLevelInner[0] = tess_level;
                    }
                ",
                ),
                tessellation_evaluation_shader: Some(
                    "
                    #version 400

                    layout(triangles, equal_spacing) in;

                    void main() {
                        vec3 position = vec3(gl_TessCoord.x) * gl_in[0].gl_Position.xyz +
                                        vec3(gl_TessCoord.y) * gl_in[1].gl_Position.xyz +
                                        vec3(gl_TessCoord.z) * gl_in[2].gl_Position.xyz;
                        gl_Position = vec4(position, 1.0);
                    }

                ",
                ),
            },
        )
        .unwrap();

        // level of tessellation
        let tess_level: i32 = 5;
        println!(
            "The current tessellation level is {} ; use the Up and Down keys to change it",
            tess_level
        );

        Self {
            vertex_buffer,
            index_buffer,
            tess_level,
            program,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let mut frame = display.draw();
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tess_level: self.tess_level
        };
        frame.clear_color(0.0, 0.0, 0.0, 0.0);
        frame
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        frame.finish().unwrap();
    }

    fn handle_window_event(&mut self, event: &winit::event::WindowEvent, _window: &winit::window::Window) {
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => match input.state {
                winit::event::ElementState::Pressed => match input.virtual_keycode {
                    Some(winit::event::VirtualKeyCode::Up) => {
                        self.tess_level += 1;
                        println!("New tessellation level: {}", self.tess_level);
                    }
                    Some(winit::event::VirtualKeyCode::Down) => {
                        if self.tess_level >= 2 {
                            self.tess_level -= 1;
                            println!("New tessellation level: {}", self.tess_level);
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
    }
}

fn main() {
    State::<Application>::run_loop();
}
