extern crate gl;
extern crate sdl2;

use gl::types::GLchar;
use gl::types::GLint;

use std::mem;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::ffi::CString;
use std::ptr;
use nalgebra_glm as glm;

const WINDOW_WIDTH: u32 = 80*23;
const WINDOW_HEIGHT: u32 = 60*23;

const VERTEX_SHADER_SRC: &'static str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec3 aColor;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 projection;

    out vec3 ourColor;

    void main() {
        gl_Position = projection * view * model * vec4(aPos, 1.0);
        ourColor = aColor;
    }
"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
    #version 330 core
    in vec3 ourColor;

    out vec4 color;

    void main() {
        color = vec4(ourColor, 1.0);
    }
"#;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Rust SDL2 OpenGL Cubes", WINDOW_WIDTH, WINDOW_HEIGHT)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    unsafe {
        gl::Viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
        gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    }

    let (shader_program, vao) = setup_cubes();

    let view = glm::look_at(
        &glm::vec3(-5.0, -5.0, 5.0),
        &glm::vec3(4.0, 4.0, 0.0),
        &glm::vec3(0.0, 0.0, 1.0),
    );

    let projection = glm::ortho(-6.0, 6.0, -6.0, 6.0, 0.1, 100.0);

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program);

            let model_cstr = CString::new("model").unwrap();
            let model_loc = gl::GetUniformLocation(shader_program, model_cstr.as_ptr());
        
            let view_cstr = CString::new("view").unwrap();
            let view_loc = gl::GetUniformLocation(shader_program, view_cstr.as_ptr());
        
            let projection_cstr = CString::new("projection").unwrap();
            let projection_loc = gl::GetUniformLocation(shader_program, projection_cstr.as_ptr());

            gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(projection_loc, 1, gl::FALSE, projection.as_ptr());

            gl::BindVertexArray(vao);

            let vertex_color_cstr = CString::new("vertexColor").unwrap();
            let vertex_color_loc = gl::GetUniformLocation(shader_program, vertex_color_cstr.as_ptr());
            
            for x in 0..8 {
                for y in 0..8 {
                    let color = if (x + y) % 2 == 0 {
                        glm::vec3(0.1, 0.1, 0.2) // bluish black
                    } else {
                        glm::vec3(0.95, 0.95, 0.85) // creamy ivory
                    };
            
                    let model = glm::translate(&glm::identity(), &glm::vec3(x as f32, y as f32, -0.5));
                    gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());
                    gl::Uniform3f(vertex_color_loc, color[0], color[1], color[2]);
            
                    gl::DrawArrays(gl::TRIANGLES, 0, 36);
                }
            }
        }

        window.gl_swap_window();
    }
}

fn setup_cubes() -> (u32, u32) {
    let vertices: [f32; 108] = [
        // Positions (xyz)
        -0.5, -0.5, -0.5,
         0.5, -0.5, -0.5,
         0.5,  0.5, -0.5,
         0.5,  0.5, -0.5,
        -0.5,  0.5, -0.5,
        -0.5, -0.5, -0.5,

        -0.5, -0.5,  0.5,
         0.5, -0.5,  0.5,
         0.5,  0.5,  0.5,
         0.5,  0.5,  0.5,
        -0.5,  0.5,  0.5,
        -0.5, -0.5,  0.5,

        -0.5,  0.5,  0.5,
        -0.5,  0.5, -0.5,
        -0.5, -0.5, -0.5,
        -0.5, -0.5, -0.5,
        -0.5, -0.5,  0.5,
        -0.5,  0.5,  0.5,

         0.5,  0.5,  0.5,
         0.5,  0.5, -0.5,
         0.5, -0.5, -0.5,
         0.5, -0.5, -0.5,
         0.5, -0.5,  0.5,
         0.5,  0.5,  0.5,

        -0.5, -0.5, -0.5,
         0.5, -0.5, -0.5,
         0.5, -0.5,  0.5,
         0.5, -0.5,  0.5,
        -0.5, -0.5,  0.5,
        -0.5, -0.5, -0.5,

        -0.5,  0.5, -0.5,
         0.5,  0.5, -0.5,
         0.5,  0.5,  0.5,
         0.5,  0.5,  0.5,
        -0.5,  0.5,  0.5,
        -0.5,  0.5, -0.5,
    ];

    let (shader_program, vao) = setup_opengl(&vertices);

    (shader_program, vao)
}

fn setup_opengl(vertices: &[f32]) -> (u32, u32) {
    let vertex_shader = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);

    let shader_program = link_program(vertex_shader, fragment_shader);

    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );

        // Set vertex attributes
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * mem::size_of::<f32>()) as gl::types::GLint,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // Set color attributes
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * mem::size_of::<f32>()) as gl::types::GLint,
            (3 * mem::size_of::<f32>()) as *const gl::types::GLvoid,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    (shader_program, vao)
}

fn compile_shader(src: &str, shader_type: u32) -> u32 {
    let shader;
    unsafe {
        shader = gl::CreateShader(shader_type);
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut log_length = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);
            let mut log = Vec::with_capacity(log_length as usize);
            log.set_len((log_length as usize) - 1);
            gl::GetShaderInfoLog(
                shader,
                log_length,
                ptr::null_mut(),
                log.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "Failed to compile shader: {}",
                std::str::from_utf8(&log).unwrap()
            );
        }
    }

    shader
}

fn link_program(vertex_shader: u32, fragment_shader: u32) -> u32 {
    let program;
    unsafe {
        program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut success = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut log_length = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);
                let mut log = Vec::with_capacity(log_length as usize);
                log.set_len((log_length as usize) - 1);
                gl::GetProgramInfoLog(
                    program,
                    log_length,
                    ptr::null_mut(),
                    log.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "Failed to link program: {}",
                    std::str::from_utf8(&log).unwrap()
                );
            }
    
            gl::DetachShader(program, vertex_shader);
            gl::DetachShader(program, fragment_shader);
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }
    
        program
    }
}
