extern crate gl;
extern crate sdl2;

use gl::types::GLchar;
use gl::types::GLint;
use sdl2::keyboard::Scancode;

use std::ffi::c_void;
use std::mem;
use std::time::Instant;

use nalgebra_glm as glm;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::ffi::CString;
use std::ptr;

const WINDOW_WIDTH: u32 = 80 * 23;
const WINDOW_HEIGHT: u32 = 60 * 23;

const VERTEX_SHADER_SRC: &'static str = r#"
#version 420 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform vec3 viewPos;
uniform vec3 lightPos;

out vec3 Normal;
out vec3 FragPos;
out vec3 LightPosition;
out vec3 ViewPosition;

void main() {
    gl_Position = projection * view * model * vec4(aPos, 1.0);
    FragPos = vec3(model * vec4(aPos, 1.0));
    Normal = mat3(transpose(inverse(model))) * aNormal;
    LightPosition = vec3(view * vec4(lightPos, 1.0));
    ViewPosition = viewPos;
}

"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
#version 420 core

in vec3 Normal;
in vec3 FragPos;

uniform vec3 objectColor;
uniform vec3 lightPos;
uniform vec3 viewPos;

out vec4 color;

void main() {
    // Ambient
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * objectColor;

    // Diffuse
    float diffuseStrength = 1.0;
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(lightPos - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * objectColor * diffuseStrength;

    // Specular
    float specularStrength = 0.8;
    vec3 viewDir = normalize(viewPos - FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 4);
    vec3 specular = specularStrength * spec * objectColor;

    // Combine results
    vec3 result = (ambient + diffuse + specular);
    color = vec4(result, 1.0);
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

        gl::Disable(gl::CULL_FACE);
    }

    let (vao, shader_program, axis_vao, _axis_vbo) = setup_rendering();

    let _aspect_ratio = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;

    let default_view = glm::look_at(
        &glm::vec3(5.0, 5.0, 10.0),
        &glm::vec3(-5.0, -5.0, -9.0),
        &glm::vec3(0.0, 0.0, 1.0),
    );

    // Create a rotation matrix for 180 degrees around the Z axis
    let rotation = glm::rotate_z(&glm::identity(), 180.0_f32.to_radians());

    // Multiply the rotation matrix with the view matrix
    let _view = rotation * default_view;

    let mut pitch = 135.0_f32;
    let roll = 0.0_f32;
    let mut yaw = -225.0_f32;

    let mut frame_counter = 0;
    let timer = Instant::now();
    let mut last_printed_second = 0;

    // let half_width = 7.0;
    // let half_height = half_width / aspect_ratio;
    // let projection = glm::ortho(-half_width, half_width, -half_height, half_height, 0.1, 100.0);
    let aspect_ratio = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;
    let fov = 45.0_f32.to_radians();
    let near = 0.1;
    let far = 100.0;
    let projection = glm::perspective(aspect_ratio, fov, near, far);

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        frame_counter += 1;
        let elapsed_seconds = timer.elapsed().as_secs();
        if elapsed_seconds > last_printed_second {
            // let fps = frame_counter as f64 / elapsed_seconds as f64;
            println!("Frames per second: {}", frame_counter);
            last_printed_second = elapsed_seconds;
            frame_counter = 0;
        }

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

        // Handle key states for continuous rotation
        let key_state = event_pump.keyboard_state();
        // if key_state.is_scancode_pressed(Scancode::Q) {
        //     yaw += 1.0;
        // }
        // if key_state.is_scancode_pressed(Scancode::E) {
        //     yaw -= 1.0;
        // }
        if key_state.is_scancode_pressed(Scancode::W) {
            if pitch < 179.0 {
                pitch += 1.0;
            }
        }
        if key_state.is_scancode_pressed(Scancode::S) {
            if pitch > 1.0 {
                pitch -= 1.0;
            }
        }
        if key_state.is_scancode_pressed(Scancode::A) {
            yaw += 1.0;
        }
        if key_state.is_scancode_pressed(Scancode::D) {
            yaw -= 1.0;
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(shader_program);

            let model_cstr = CString::new("model").unwrap();
            let model_loc = gl::GetUniformLocation(shader_program, model_cstr.as_ptr());

            let color_cstr = CString::new("objectColor").unwrap();
            let color_loc = gl::GetUniformLocation(shader_program, color_cstr.as_ptr());

            let camera_distance = 15.0_f32;

            // Update view matrix based on the updated rotations
            let rotation_matrix = glm::rotate_z(&glm::identity(), yaw.to_radians())
                * glm::rotate_x(&glm::identity(), pitch.to_radians())
                * glm::rotate_y(&glm::identity(), roll.to_radians());

            let camera_position = rotation_matrix * glm::vec4(0.0, 0.0, -camera_distance, 1.0);

            let light_pos_cstr = CString::new("lightPos").unwrap();
            let light_pos_loc = gl::GetUniformLocation(shader_program, light_pos_cstr.as_ptr());
            gl::Uniform3f(
                light_pos_loc,
                camera_position.x,
                camera_position.y,
                camera_position.z,
            );

            let view = glm::look_at(
                &glm::vec3(camera_position.x, camera_position.y, camera_position.z),
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(0.0, 0.0, 1.0),
            );

            // println!("yaw: {}, pitch: {}, roll: {}", yaw, pitch, roll);

            let view_cstr = CString::new("view").unwrap();
            let view_loc = gl::GetUniformLocation(shader_program, view_cstr.as_ptr());

            let projection_cstr = CString::new("projection").unwrap();
            let projection_loc = gl::GetUniformLocation(shader_program, projection_cstr.as_ptr());

            gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(projection_loc, 1, gl::FALSE, projection.as_ptr());

            gl::BindVertexArray(vao);

            // let vertex_color_cstr = CString::new("vertexColor").unwrap();
            // let vertex_color_loc = gl::GetUniformLocation(shader_program, vertex_color_cstr.as_ptr());

            let camera_pos_loc =
                gl::GetUniformLocation(shader_program, b"cameraPos\0".as_ptr() as *const i8);
            gl::Uniform3fv(camera_pos_loc, 1, camera_position.as_ptr());

            for x in -3..5 {
                for y in -3..5 {
                    let color = if (x + y) % 2 == 0 {
                        glm::vec3(0.1, 0.1, 0.2) // bluish black
                    } else {
                        glm::vec3(0.95, 0.95, 0.85) // creamy ivory
                    };

                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                    let model = glm::translate(
                        &glm::identity(),
                        &glm::vec3(x as f32 - 0.5, y as f32 - 0.5, 0.0),
                    );
                    gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());
                    gl::Uniform3fv(color_loc, 1, color.as_ptr());

                    gl::DrawArrays(gl::TRIANGLES, 0, 36);

                    // wireframe
                    // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                    // gl::LineWidth(1.0);
                    // gl::Uniform3f(color_loc, 0.0, 0.0, 0.0);

                    // gl::DrawArrays(gl::TRIANGLES, 0, 36);
                }
            }

            // // Reset the model matrix to the identity matrix
            // let identity: glm::Mat4 = glm::identity();
            // gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, identity.as_ptr());

            // // Draw axis lines
            // gl::BindVertexArray(axis_vao);
            // gl::LineWidth(2.0);

            // // X-axis (red)
            // gl::Uniform3f(color_loc, 1.0, 0.0, 0.0);
            // gl::DrawArrays(gl::LINES, 0, 2);

            // // Y-axis (green)
            // gl::Uniform3f(color_loc, 0.0, 1.0, 0.0);
            // gl::DrawArrays(gl::LINES, 2, 2);

            // // Z-axis (blue)
            // gl::Uniform3f(color_loc, 0.0, 0.0, 1.0);
            // gl::DrawArrays(gl::LINES, 4, 2);
        }

        window.gl_swap_window();
    }
}

fn setup_rendering() -> (u32, u32, u32, u32) {
    let cube_vertices: [f32; 216] = [
        // Front face
        -0.5, -0.5, 0.5, 0.0, 0.0, 0.1, //v1
        0.5, 0.5, 0.5, 0.0, 0.0, 0.1, // v2
        0.5, -0.5, 0.5, 0.0, 0.0, 0.1, // v3
        -0.5, 0.5, 0.5, 0.0, 0.0, 0.1, // v4
        0.5, 0.5, 0.5, 0.0, 0.0, 0.1, // v5
        -0.5, -0.5, 0.5, 0.0, 0.0, 0.1, // v6
        // Right face
        0.5, -0.5, 0.5, 1.0, 0.0, 0.0, // v7
        0.5, 0.5, -0.5, 1.0, 0.0, 0.0, // v8
        0.5, -0.5, -0.5, 1.0, 0.0, 0.0, // v9
        0.5, 0.5, -0.5, 1.0, 0.0, 0.0, // v10
        0.5, 0.5, 0.5, 1.0, 0.0, 0.0, // v11
        0.5, -0.5, 0.5, 1.0, 0.0, 0.0, // v12
        // Back face
        0.5, -0.5, -0.5, 0.0, 0.0, -1.0, // v13
        -0.5, 0.5, -0.5, 0.0, 0.0, -1.0, // v14
        -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, // v15
        -0.5, 0.5, -0.5, 0.0, 0.0, -1.0, // v16
        0.5, 0.5, -0.5, 0.0, 0.0, -1.0, // v17
        0.5, -0.5, -0.5, 0.0, 0.0, -1.0, // v18
        // Left face
        -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, // v19
        -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, // v20
        -0.5, -0.5, 0.5, -1.0, 0.0, 0.0, // v21
        -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, // v22
        -0.5, 0.5, -0.5, -1.0, 0.0, 0.0, // v23
        -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, // v24
        // Top face
        -0.5, 0.5, 0.5, 0.0, 1.0, 0.0, // v25
        0.5, 0.5, -0.5, 0.0, 1.0, 0.0, // v26
        0.5, 0.5, 0.5, 0.0, 1.0, 0.0, // v27
        0.5, 0.5, -0.5, 0.0, 1.0, 0.0, // v28
        -0.5, 0.5, -0.5, 0.0, 1.0, 0.0, // v29
        -0.5, 0.5, 0.5, 0.0, 1.0, 0.0, // v30
        // Bottom face
        -0.5, -0.5, 0.5, 0.0, -1.0, 0.0, // v31
        0.5, -0.5, -0.5, 0.0, -1.0, 0.0, // v32
        -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, // v33
        0.5, -0.5, -0.5, 0.0, -1.0, 0.0, // v34
        0.5, -0.5, 0.5, 0.0, -1.0, 0.0, // v35
        -0.5, -0.5, 0.5, 0.0, -1.0, 0.0, // v36
    ];

    let axis_vertices: [f32; 18] = [
        0.0, 0.0, 0.0, // X-axis origin
        10.0, 0.0, 0.0, // X-axis length
        0.0, 0.0, 0.0, // Y-axis origin
        0.0, 10.0, 0.0, // Y-axis length
        0.0, 0.0, 0.0, // Z-axis origin
        0.0, 0.0, 10.0, // Z-axis length
    ];

    let (vao, shader_program, axis_vao, axis_vbo) = setup_opengl(&cube_vertices, &axis_vertices);

    (vao, shader_program, axis_vao, axis_vbo)
}

fn setup_opengl(cube_vertices: &[f32], axis_vertices: &[f32]) -> (u32, u32, u32, u32) {
    let vertex_shader = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);

    let shader_program = link_program(vertex_shader, fragment_shader);

    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
        // gl::Disable(gl::CULL_FACE);
        // Enable depth testing
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);

        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (cube_vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            cube_vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<f32>() as i32,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<f32>() as i32,
            (3 * mem::size_of::<f32>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    let mut axis_vao = 0;
    let mut axis_vbo = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut axis_vao);
        gl::GenBuffers(1, &mut axis_vbo);

        gl::BindVertexArray(axis_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, axis_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (axis_vertices.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            axis_vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * mem::size_of::<f32>()) as gl::types::GLint,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
    }

    (vao, shader_program, axis_vao, axis_vbo)
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
