// main.rs - Solar System Simulation with Spaceship
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

// Estructura para representar un cuerpo celeste
struct CelestialBody {
    planet_type: i32,
    orbital_radius: f32,
    orbital_speed: f32,
    rotation_speed: f32,
    scale: f32,
    orbital_angle: f32,
    rotation_angle: f32,
    name: &'static str,
}

impl CelestialBody {
    fn new(planet_type: i32, orbital_radius: f32, orbital_speed: f32,
           rotation_speed: f32, scale: f32, name: &'static str) -> Self {
        CelestialBody {
            planet_type,
            orbital_radius,
            orbital_speed,
            rotation_speed,
            scale,
            orbital_angle: 0.0,
            rotation_angle: 0.0,
            name,
        }
    }

    fn update(&mut self, dt: f32) {
        self.orbital_angle += self.orbital_speed * dt;
        self.rotation_angle += self.rotation_speed * dt;
    }

    fn get_position(&self) -> Vector3 {
        Vector3::new(
            self.orbital_radius * self.orbital_angle.cos(),
            0.0,
            self.orbital_radius * self.orbital_angle.sin(),
        )
    }

    fn get_orbit_points(&self, segments: usize) -> Vec<Vector3> {
        let mut points = Vec::new();
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * 2.0 * PI;
            points.push(Vector3::new(
                self.orbital_radius * angle.cos(),
                0.0,
                self.orbital_radius * angle.sin(),
            ));
        }
        points
    }
}

fn render_body(framebuffer: &mut Framebuffer, uniforms: &Uniforms,
               vertex_array: &[Vertex], light: &Light) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());

    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

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

    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], light));
    }

    for fragment in fragments {
        let final_color = fragment_shader(&fragment, uniforms);
        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            final_color,
            fragment.depth,
        );
    }
}

fn render_orbit(_framebuffer: &mut Framebuffer, _points: &[Vector3],
                _view_matrix: &Matrix, _projection_matrix: &Matrix,
                _viewport_matrix: &Matrix, _color: Color) {
    // Esta función renderiza las órbitas como líneas
    // Necesitarías implementar esto según tu sistema de proyección
    // Por ahora es un placeholder
}

fn main() {
    let window_width = 1600;
    let window_height = 900;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Sistema Solar con Nave - Software Renderer")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width, window_height);

    // Cámara inicial
    let mut camera = Camera::new(
        Vector3::new(0.0, 15.0, 25.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    // Luz desde el sol
    let light = Light::new(Vector3::new(0.0, 0.0, 0.0));

    // Cargar modelos
    let sphere_obj = Obj::load("models/sphere.obj").expect("Failed to load sphere.obj");
    let sphere_vertex_array = sphere_obj.get_vertex_array();
    
    let nave_obj = Obj::load("models/nave.obj").expect("Failed to load nave.obj");
    let nave_vertex_array = nave_obj.get_vertex_array();

    framebuffer.set_background_color(Color::new(5, 5, 15, 255));

    // Crear el sistema solar
    let mut sun = CelestialBody::new(5, 0.0, 0.0, 0.1, 2.5, "Sol");

    let mut planets = vec![
        CelestialBody::new(0, 5.0, 0.8, 2.0, 0.6, "Mercurio"),    // Rocky
        CelestialBody::new(1, 8.0, 0.6, 1.5, 0.9, "Venus"),       // Gaseous
        CelestialBody::new(2, 12.0, 0.5, 1.8, 1.0, "Tierra"),     // Custom
        CelestialBody::new(3, 18.0, 0.3, 1.2, 1.3, "Saturno"),    // Con anillos
        CelestialBody::new(4, 24.0, 0.2, 0.9, 1.1, "Neptuno"),    // Extra planet
    ];

    // Nave espacial - posicionada en la cámara
    let nave_scale = 0.3;  // Tamaño visible
    let nave_offset = Vector3::new(0.8, -0.5, -3.0);  // Offset desde la cámara (más lejos hacia adelante)

    let mut time = 0.0;
    let mut warp_target: Option<usize> = None;
    let mut warp_progress = 0.0;
    let mut show_orbits = true;
    let mut camera_mode = 0; // 0: free, 1-5: following planets

    println!("=== Controles ===");
    println!("WASD/Flechas: Mover cámara");
    println!("Q/E: Subir/Bajar cámara");
    println!("1-5: Seguir planetas");
    println!("0: Cámara libre");
    println!("SPACE: Warp al siguiente planeta");
    println!("O: Toggle órbitas");
    println!("R: Reset cámara");

    while !window.window_should_close() {
        let dt = window.get_frame_time();
        time += dt;

        // Actualizar cuerpos celestes
        sun.update(dt);
        for planet in &mut planets {
            planet.update(dt);
        }

        // Input handling
        if window.is_key_pressed(KeyboardKey::KEY_ZERO) {
            camera_mode = 0;
            warp_target = None;
        }
        for i in 0..5 {
            if window.is_key_pressed(match i {
                0 => KeyboardKey::KEY_ONE,
                1 => KeyboardKey::KEY_TWO,
                2 => KeyboardKey::KEY_THREE,
                3 => KeyboardKey::KEY_FOUR,
                _ => KeyboardKey::KEY_FIVE,
            }) {
                warp_target = Some(i);
                warp_progress = 0.0;
            }
        }

        if window.is_key_pressed(KeyboardKey::KEY_SPACE) {
            if let Some(current) = warp_target {
                warp_target = Some((current + 1) % planets.len());
            } else {
                warp_target = Some(0);
            }
            warp_progress = 0.0;
        }

        if window.is_key_pressed(KeyboardKey::KEY_O) {
            show_orbits = !show_orbits;
        }

        if window.is_key_pressed(KeyboardKey::KEY_R) {
            camera = Camera::new(
                Vector3::new(0.0, 15.0, 25.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            );
            camera_mode = 0;
            warp_target = None;
        }

        // Warp animation - actualizar primero
        if let Some(target_idx) = warp_target {
            warp_progress += dt * 2.0;
            if warp_progress >= 1.0 {
                warp_progress = 1.0;
                camera_mode = target_idx + 1;
                warp_target = None;
            }
        }

        // Camera control
        if camera_mode > 0 && camera_mode <= 5 {
            let planet_idx = camera_mode - 1;
            let planet_pos = planets[planet_idx].get_position();

            // La cámara sigue al planeta
            camera.target = planet_pos;
            camera.distance = 5.0;

            // Permitir rotación alrededor del planeta
            if window.is_key_down(KeyboardKey::KEY_A) {
                camera.yaw += camera.rotation_speed;
            }
            if window.is_key_down(KeyboardKey::KEY_D) {
                camera.yaw -= camera.rotation_speed;
            }
            if window.is_key_down(KeyboardKey::KEY_W) {
                camera.pitch += camera.rotation_speed;
            }
            if window.is_key_down(KeyboardKey::KEY_S) {
                camera.pitch -= camera.rotation_speed;
            }

            camera.pitch = camera.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
            camera.eye.x = camera.target.x + camera.distance * camera.pitch.cos() * camera.yaw.cos();
            camera.eye.y = camera.target.y + camera.distance * camera.pitch.sin();
            camera.eye.z = camera.target.z + camera.distance * camera.pitch.cos() * camera.yaw.sin();
        } else {
            camera.process_input(&window);
        }

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

        // Renderizar órbitas
        if show_orbits {
            for planet in &planets {
                let orbit_points = planet.get_orbit_points(64);
                render_orbit(
                    &mut framebuffer,
                    &orbit_points,
                    &view_matrix,
                    &projection_matrix,
                    &viewport_matrix,
                    Color::new(100, 100, 150, 100)
                );
            }
        }

        // Renderizar el Sol
        let sun_pos = sun.get_position();
        let sun_rotation = Vector3::new(0.0, sun.rotation_angle, 0.0);
        let sun_model_matrix = create_model_matrix(sun_pos, sun.scale, sun_rotation);

        let sun_uniforms = Uniforms {
            model_matrix: sun_model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            dt,
            planet_type: sun.planet_type,
            render_type: 0,
        };
        render_body(&mut framebuffer, &sun_uniforms, &sphere_vertex_array, &light);

        // Renderizar planetas
        for planet in &planets {
            let planet_pos = planet.get_position();
            let planet_rotation = Vector3::new(0.0, planet.rotation_angle, 0.0);
            let planet_model_matrix = create_model_matrix(
                planet_pos,
                planet.scale,
                planet_rotation
            );

            let planet_uniforms = Uniforms {
                model_matrix: planet_model_matrix,
                view_matrix,
                projection_matrix,
                viewport_matrix,
                time,
                dt,
                planet_type: planet.planet_type,
                render_type: 0,
            };
            render_body(&mut framebuffer, &planet_uniforms, &sphere_vertex_array, &light);

            // Renderizar anillos si es Saturno (tipo 3)
            if planet.planet_type == 3 {
                // Llamar a render_rings con las transformaciones apropiadas
            }
        }

        // Renderizar la nave espacial (pegada a la cámara, enfrente)
        // Calcular la dirección hacia donde mira la cámara
        let camera_forward = Vector3::new(
            camera.target.x - camera.eye.x,
            camera.target.y - camera.eye.y,
            camera.target.z - camera.eye.z,
        );
        let forward_length = (camera_forward.x * camera_forward.x + 
                             camera_forward.y * camera_forward.y + 
                             camera_forward.z * camera_forward.z).sqrt();
        let camera_forward = Vector3::new(
            camera_forward.x / forward_length,
            camera_forward.y / forward_length,
            camera_forward.z / forward_length,
        );
        
        // Vector derecho de la cámara
        let camera_right = Vector3::new(
            camera_forward.z,
            0.0,
            -camera_forward.x,
        );
        let right_length = (camera_right.x * camera_right.x + 
                           camera_right.z * camera_right.z).sqrt();
        let camera_right = Vector3::new(
            camera_right.x / right_length,
            camera_right.y,
            camera_right.z / right_length,
        );
        
        let camera_up = Vector3::new(0.0, 1.0, 0.0);
        
        // Posición de la nave ENFRENTE de la cámara
        let nave_position = Vector3::new(
            camera.eye.x + camera_forward.x * (-nave_offset.z) + camera_right.x * nave_offset.x + camera_up.x * nave_offset.y,
            camera.eye.y + camera_forward.y * (-nave_offset.z) + camera_right.y * nave_offset.x + camera_up.y * nave_offset.y,
            camera.eye.z + camera_forward.z * (-nave_offset.z) + camera_right.z * nave_offset.x + camera_up.z * nave_offset.y,
        );
        
        // Rotación de la nave para que apunte hacia adelante
        let nave_rotation = Vector3::new(0.0, camera.yaw + PI, 0.0);
        
        let nave_model_matrix = create_model_matrix(
            nave_position,
            nave_scale,
            nave_rotation
        );

        let nave_uniforms = Uniforms {
            model_matrix: nave_model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            dt,
            planet_type: 10, // Tipo especial para la nave
            render_type: 0,
        };
        render_body(&mut framebuffer, &nave_uniforms, &nave_vertex_array, &light);

        // UI Info
        let info_text = format!(
            "FPS: {:.0} | Modo: {} | Órbitas: {}",
            1.0 / dt,
            if camera_mode == 0 { "Libre".to_string() }
            else { planets[camera_mode - 1].name.to_string() },
            if show_orbits { "ON" } else { "OFF" }
        );

        framebuffer.swap_buffers(&mut window, &raylib_thread);

        let mut d = window.begin_drawing(&raylib_thread);
        d.draw_text(&info_text, 10, 10, 20, Color::WHITE);

        thread::sleep(Duration::from_millis(16));
    }
}
