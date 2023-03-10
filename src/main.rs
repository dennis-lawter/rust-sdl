extern crate gl;
extern crate sdl2;

use gl::types::GLfloat;
use sdl2::event::Event;
use sdl2::video::GLProfile;
use std::ffi::CString;
use std::os::raw::c_void;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() {
    // Initialize SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // Create a window
    let window = video_subsystem
        .window("Grid of Cubes", WINDOW_WIDTH, WINDOW_HEIGHT)
        .opengl()
        .build()
        .unwrap();

    // Create an OpenGL context
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    // Set up the viewport
    unsafe {
        gl::Viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
    }

    // Load and compile the shaders
    let vertex_shader_source = CString::new(include_str!("vertex.glsl")).unwrap();
    let shader_program = unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            vertex_shader,
            1,
            &vertex_shader_source.as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(vertex_shader);

        let fragment_shader_source = CString::new(include_str!("fragment.glsl")).unwrap();
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            fragment_shader,
            1,
            &fragment_shader_source.as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(fragment_shader);

        // Create the shader program
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        let color_location = gl::GetUniformLocation(shader_program, "color\0".as_ptr() as *const i8);
        let color: [f32; 4] = [1.0, 0.0, 0.0, 1.0]; // red color with alpha 1.0
        gl::Uniform4fv(color_location, 1, color.as_ptr());
        
        shader_program
    };

    // Set up the cube vertices
    let cube_vertices: [f32; 108] = [
        -1.0,-1.0,-1.0, // triangle 1 : begin
        -1.0,-1.0, 1.0,
        -1.0, 1.0, 1.0, // triangle 1 : end
        1.0, 1.0,-1.0, // triangle 2 : begin
        -1.0,-1.0,-1.0,
        -1.0, 1.0,-1.0, // triangle 2 : end
        1.0,-1.0, 1.0,
        -1.0,-1.0,-1.0,
        1.0,-1.0,-1.0,
        1.0, 1.0,-1.0,
        1.0,-1.0,-1.0,
        -1.0,-1.0,-1.0,
        -1.0,-1.0,-1.0,
        -1.0, 1.0, 1.0,
        -1.0, 1.0,-1.0,
        1.0,-1.0, 1.0,
        -1.0,-1.0, 1.0,
        -1.0,-1.0,-1.0,
        -1.0, 1.0, 1.0,
        -1.0,-1.0, 1.0,
        1.0,-1.0, 1.0,
        1.0, 1.0, 1.0,
        1.0,-1.0,-1.0,
        1.0, 1.0,-1.0,
        1.0,-1.0,-1.0,
        1.0, 1.0, 1.0,
        1.0,-1.0, 1.0,
        1.0, 1.0, 1.0,
        1.0, 1.0,-1.0,
        -1.0, 1.0,-1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0,-1.0,
        -1.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, 1.0,
        1.0,-1.0, 1.0
    ];

    // Create vertex buffer object and copy data into it
    let vbo = unsafe {
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (cube_vertices.len() * std::mem::size_of::<GLfloat>()) as isize,
            &cube_vertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        vbo
    };

    // Set up vertex array object
    let vao = unsafe {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * std::mem::size_of::<GLfloat>() as i32,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * std::mem::size_of::<GLfloat>() as i32,
            (3 * std::mem::size_of::<GLfloat>()) as *const c_void,
        );
        vao
    };

    // Set up projection matrix
    let projection = glam::Mat4::perspective_rh(
        std::f32::consts::FRAC_PI_3, // 60 degrees in radians
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        0.1,
        100.0,
    );

    // Set up camera matrix
    let camera_position = glam::vec3(5.0, 5.0, 5.0);
    let camera_target = glam::vec3(0.0, 0.0, 0.0);
    let camera_up = glam::vec3(0.0, 1.0, 0.0);
    let view = glam::Mat4::look_at_rh(
        camera_position,
        camera_target,
        camera_up,
    );

    // Get location of uniform variables in shaders
    let model_location =
        unsafe { gl::GetUniformLocation(shader_program, "model\0".as_ptr() as *const i8) };
    let view_location =
        unsafe { gl::GetUniformLocation(shader_program, "view\0".as_ptr() as *const i8) };
    let projection_location =
        unsafe { gl::GetUniformLocation(shader_program, "projection\0".as_ptr() as *const i8) };

    // Main loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut running = true;
    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                _ => {}
            }
        }

        // Clear screen
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            // gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Draw cubes
        for x in -5..5 {
            for y in -5..5 {
                let model = glam::Mat4::from_translation(glam::Vec3::new(x as f32, y as f32, 0.0))
                    * glam::Mat4::IDENTITY;

                let model_view_projection = projection * view * model;

                unsafe {
                    gl::UniformMatrix4fv(
                        model_location,
                        1,
                        gl::FALSE,
                        model_view_projection.to_cols_array().as_ptr(),
                    );
                    gl::UniformMatrix4fv(
                        view_location,
                        1,
                        gl::FALSE,
                        view.to_cols_array().as_ptr(),
                    );
                    gl::UniformMatrix4fv(
                        projection_location,
                        1,
                        gl::FALSE,
                        projection.to_cols_array().as_ptr(),
                    );
                }
                unsafe {
                    gl::DrawArrays(gl::TRIANGLES, 0, 36);
                }
            }
        }

        // Swap buffers
        window.gl_swap_window();
    }

    // Cleanup
    unsafe {
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteProgram(shader_program);
    }
}
