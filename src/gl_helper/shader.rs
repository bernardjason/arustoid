use std::{ptr, str};
use std::ffi::CString;

use crate::gl;

pub fn create_shader(gl: &gl::Gl, image_vertex_shader_source:&str, image_fragment_shader_source:&str) -> u32 {
    unsafe {
        let vertex_shader = gl.CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(image_vertex_shader_source.as_bytes()).unwrap();
        gl.ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl.CompileShader(vertex_shader);


        let mut success = gl::FALSE as gl::types::GLint;
        let mut info_log = Vec::with_capacity(2048);
        info_log.set_len(2048 - 1); // subtract 1 to skip the trailing null character
        gl.GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as gl::types::GLint {
            gl.GetShaderInfoLog(vertex_shader, 2048, ptr::null_mut(), info_log.as_mut_ptr() as *mut gl::types::GLchar);
            println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }

        let fragment_shader = gl.CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(image_fragment_shader_source.as_bytes()).unwrap();
        gl.ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl.CompileShader(fragment_shader);
        gl.GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as gl::types::GLint {
            gl.GetShaderInfoLog(fragment_shader, 2048, ptr::null_mut(), info_log.as_mut_ptr() as *mut gl::types::GLchar);
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }

        let shader_program = gl.CreateProgram();
        gl.AttachShader(shader_program, vertex_shader);
        gl.AttachShader(shader_program, fragment_shader);
        gl.LinkProgram(shader_program);
        gl.GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as gl::types::GLint {
            gl.GetProgramInfoLog(shader_program, 2048, ptr::null_mut(), info_log.as_mut_ptr() as *mut gl::types::GLchar);
            println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }
        gl.DeleteShader(vertex_shader);
        gl.DeleteShader(fragment_shader);
        shader_program
    }
}