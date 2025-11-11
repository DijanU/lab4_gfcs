// main.rs - Simplified to render a single spaceship
mod framebuffer;
mod triangle;
mod obj;
mod matrix;
mod fragment;
mod vertex;
mod camera;
mod shaders;
mod light;

use triangle::triangle;
use obj::Obj;
use framebuffer::Framebuffer;
use raylib::prelude::*;
use std::thread;
use std::time::Duration;
use std::f32::consts::PI;
use matrix::{create_model_matrix, create_projection_matrix, create_viewport_matrix};
use vertex::Vertex;
use camera::Camera;
use shaders::{vertex_shader, fragment_shader};
use light::Light;
use rayon::prelude::*;

#[derive(Clone)]
pub struct Uniforms {
    pub model_matrix: Matrix,
    pub view_matrix: Matrix,
    pub projection_matrix: Matrix,
    pub viewport_matrix: Matrix,
    pub time: f32,
    pub dt: f32,
    pub planet_type: i32,
    pub render_type: i32,
}

fn render_body(framebuffer: &mut Framebuffer, uniforms: &Uniforms,
               vertex_array: &[Vertex], light: &Light) {
    // Vertex shader - paralelizado
    let transformed_vertices: Vec<Vertex> = vertex_array
        .par_iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // Crear triángulos
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Frustum culling, back-face culling, and rasterization - parallelized
    let all_fragments: Vec<Vec<_>> = triangles
        .par_iter()
        .filter_map(|tri| {
            let v0 = &tri[0].transformed_position;
            let v1 = &tri[1].transformed_position;
            let v2 = &tri[2].transformed_position;

            // Frustum culling - check near and far planes
            if (v0.z < 0.1 && v1.z < 0.1 && v2.z < 0.1) || 
               (v0.z > 200.0 && v1.z > 200.0 && v2.z > 200.0) {
                return None;
            }

            // Back-face culling
            let edge1 = Vector3::new(v1.x - v0.x, v1.y - v0.y, 0.0);
            let edge2 = Vector3::new(v2.x - v0.x, v2.y - v0.y, 0.0);
            
            let normal_z = edge1.x * edge2.y - edge1.y * edge2.x;

            if normal_z < 0.0 {
                return None; // Triangle is facing away from the camera
            }
            
            Some(triangle(&tri[0], &tri[1], &tri[2], light))
        })
        .collect();

    // Flatten todos los fragmentos
    let fragments: Vec<_> = all_fragments.into_iter().flatten().collect();

    // Dibujar fragmentos
    for fragment in fragments {
        let x = fragment.position.x as i32;
        let y = fragment.position.y as i32;
        
        // Screen bounds check antes de dibujar
        if x >= 0 && x < framebuffer.width as i32 && 
           y >= 0 && y < framebuffer.height as i32 {
            let final_color = shaders::fragment_shader(&fragment, uniforms);
            framebuffer.point(x, y, final_color, fragment.depth);
        }
    }
}

fn main() {
    let window_width = 1600;
    let window_height = 900;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Nave Espacial - Software Renderer")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width, window_height);

    // Cámara inicial
    let mut camera = Camera::new(
        Vector3::new(0.0, 5.0, 20.0), // eye
        Vector3::new(0.0, 0.0, 0.0), // target
        Vector3::new(0.0, 1.0, 0.0), // up
    );

    // Luz
    let light = Light::new(Vector3::new(0.0, 10.0, 10.0));

    // Cargar modelo de la esfera
    let sphere_obj = Obj::load("models/sphere.obj").expect("Failed to load sphere.obj");
    let sphere_vertex_array = sphere_obj.get_vertex_array();

    framebuffer.set_background_color(Color::new(5, 5, 15, 255));

    let mut time = 0.0;

    println!("=== Controles ===");
    println!("WASD/Flechas: Mover cámara");
    println!("Q/E: Subir/Bajar cámara");
    println!("R: Reset cámara");

    while !window.window_should_close() {
        let dt = window.get_frame_time();
        time += dt;

        // Input handling
        if window.is_key_pressed(KeyboardKey::KEY_R) {
            camera = Camera::new(
                Vector3::new(0.0, 5.0, 10.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            );
        }

        camera.process_input(&window);

        framebuffer.clear();

        // Matrices de transformación
        let view_matrix = camera.get_view_matrix();
        let projection_matrix = create_projection_matrix(
            PI / 3.0,
            window_width as f32 / window_height as f32,
            0.1,
            200.0
        );
        let viewport_matrix = create_viewport_matrix(
            0.0, 0.0,
            window_width as f32,
            window_height as f32
        );

        // Renderizar el sol
        let sun_rotation = Vector3::new(0.0, time * 0.1, 0.0); // Rotar el sol lentamente
        let sun_model_matrix = create_model_matrix(
            Vector3::new(0.0, 0.0, 0.0), // Centrada en el origen
            3.0, // Escala grande
            sun_rotation
        );

        let sun_uniforms = Uniforms {
            model_matrix: sun_model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            dt,
            planet_type: 5, // Tipo para el sol
            render_type: 3,
        };
        render_body(&mut framebuffer, &sun_uniforms, &sphere_vertex_array, &light);

        // UI Info
        let info_text = format!("FPS: {:.0}", 1.0 / dt);

        framebuffer.swap_buffers(&mut window, &raylib_thread);

        let mut d = window.begin_drawing(&raylib_thread);
        d.draw_text(&info_text, 10, 10, 20, Color::WHITE);

        thread::sleep(Duration::from_millis(16));
    }
}