// shaders.rs
use raylib::prelude::*;
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::matrix::multiply_matrix_vector4;
use crate::fragment::Fragment;
use crate::framebuffer::Framebuffer;
use crate::triangle;
use crate::light::Light;

// ============================================================================
// VERTEX SHADER
// ============================================================================
pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let mut position_vec4 = Vector4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    // Modificar geometría según el tipo de render
    match uniforms.render_type {
        1 => { // Anillos procedurales
            // Generar geometría de anillos usando las coordenadas del vértice
            let angle = vertex.position.x.atan2(vertex.position.z);
            let inner_radius = 1.8;
            let outer_radius = 2.8;
            let ring_width = outer_radius - inner_radius;
            
            // Usar Y del vértice para interpolar entre radio interno y externo
            let radius = inner_radius + (vertex.position.y + 1.0) * 0.5 * ring_width;
            
            position_vec4.x = radius * angle.cos();
            position_vec4.z = radius * angle.sin();
            position_vec4.y = vertex.position.z * 0.05; // Anillo delgado con ligera ondulación
        }
        2 => { // Luna orbital
            let moon_orbit_time = uniforms.time * 0.3;
            let moon_distance = 4.5;
            let moon_inclination = 0.2; // Inclinación orbital
            
            let moon_x = moon_distance * moon_orbit_time.cos();
            let moon_z = moon_distance * moon_orbit_time.sin();
            let moon_y = (moon_orbit_time * 2.0).sin() * moon_inclination;
            
            let moon_scale = 0.25; // Tamaño de la luna
            position_vec4.x = moon_x + vertex.position.x * moon_scale;
            position_vec4.y = moon_y + vertex.position.y * moon_scale;
            position_vec4.z = moon_z + vertex.position.z * moon_scale;
        }
        _ => {} // Planeta normal - sin deformación
    }

    // Pipeline de transformación estándar
    let world_position = multiply_matrix_vector4(&uniforms.model_matrix, &position_vec4);
    let view_position = multiply_matrix_vector4(&uniforms.view_matrix, &world_position);
    let clip_position = multiply_matrix_vector4(&uniforms.projection_matrix, &view_position);

    // Perspectiva dividida
    let ndc = if clip_position.w != 0.0 {
        Vector3::new(
            clip_position.x / clip_position.w,
            clip_position.y / clip_position.w,
            clip_position.z / clip_position.w,
        )
    } else {
        Vector3::new(clip_position.x, clip_position.y, clip_position.z)
    };

    // Viewport transformation
    let ndc_vec4 = Vector4::new(ndc.x, ndc.y, ndc.z, 1.0);
    let screen_position = multiply_matrix_vector4(&uniforms.viewport_matrix, &ndc_vec4);

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vector3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transform_normal(&vertex.normal, &uniforms.model_matrix),
    }
}

fn transform_normal(normal: &Vector3, model_matrix: &Matrix) -> Vector3 {
    let normal_vec4 = Vector4::new(normal.x, normal.y, normal.z, 0.0);
    let transformed = multiply_matrix_vector4(model_matrix, &normal_vec4);
    let mut result = Vector3::new(transformed.x, transformed.y, transformed.z);
    result.normalize();
    result
}

// ============================================================================
// FUNCIONES DE RUIDO PROCEDURAL
// ============================================================================

// Hash function para generar valores pseudo-aleatorios
fn hash(n: f32) -> f32 {
    ((n * 12.9898).sin() * 43758.5453).fract()
}

// Noise 3D mejorado
fn noise3d(p: &Vector3) -> f32 {
    let i = Vector3::new(p.x.floor(), p.y.floor(), p.z.floor());
    let f = Vector3::new(p.x.fract(), p.y.fract(), p.z.fract());
    
    // Suavizado cúbico
    let u = Vector3::new(f.x * f.x * (3.0 - 2.0 * f.x), 
                         f.y * f.y * (3.0 - 2.0 * f.y),
                         f.z * f.z * (3.0 - 2.0 * f.z));
    
    // Interpolación de las 8 esquinas del cubo
    let n000 = hash(i.x + i.y * 57.0 + i.z * 113.0);
    let n100 = hash(i.x + 1.0 + i.y * 57.0 + i.z * 113.0);
    let n010 = hash(i.x + (i.y + 1.0) * 57.0 + i.z * 113.0);
    let n110 = hash(i.x + 1.0 + (i.y + 1.0) * 57.0 + i.z * 113.0);
    let n001 = hash(i.x + i.y * 57.0 + (i.z + 1.0) * 113.0);
    let n101 = hash(i.x + 1.0 + i.y * 57.0 + (i.z + 1.0) * 113.0);
    let n011 = hash(i.x + (i.y + 1.0) * 57.0 + (i.z + 1.0) * 113.0);
    let n111 = hash(i.x + 1.0 + (i.y + 1.0) * 57.0 + (i.z + 1.0) * 113.0);
    
    let nx00 = n000 * (1.0 - u.x) + n100 * u.x;
    let nx10 = n010 * (1.0 - u.x) + n110 * u.x;
    let nx01 = n001 * (1.0 - u.x) + n101 * u.x;
    let nx11 = n011 * (1.0 - u.x) + n111 * u.x;
    
    let nxy0 = nx00 * (1.0 - u.y) + nx10 * u.y;
    let nxy1 = nx01 * (1.0 - u.y) + nx11 * u.y;
    
    nxy0 * (1.0 - u.z) + nxy1 * u.z
}

// Fractal Brownian Motion - múltiples octavas de ruido
fn fbm(p: &Vector3, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    
    for _ in 0..octaves {
        value += noise3d(&Vector3::new(p.x * frequency, p.y * frequency, p.z * frequency)) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    value
}

// Turbulencia - valor absoluto del FBM
fn turbulence(p: &Vector3, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    
    for _ in 0..octaves {
        value += noise3d(&Vector3::new(p.x * frequency, p.y * frequency, p.z * frequency)).abs() * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    value
}

// ============================================================================
// UTILIDADES
// ============================================================================

fn lerp_color(a: &Vector3, b: &Vector3, t: f32) -> Vector3 {
    let t_clamped = t.max(0.0).min(1.0);
    Vector3::new(
        a.x + (b.x - a.x) * t_clamped,
        a.y + (b.y - a.y) * t_clamped,
        a.z + (b.z - a.z) * t_clamped
    )
}

// ============================================================================
// SISTEMA DE ILUMINACIÓN
// ============================================================================

fn calculate_lighting(normal: &Vector3, light_dir: &Vector3, view_dir: &Vector3) -> (f32, f32) {
    // Normalizar vectores
    let mut n = *normal;
    n.normalize();
    let mut l = *light_dir;
    l.normalize();
    let mut v = *view_dir;
    v.normalize();
    
    // Luz difusa (Lambertian)
    let diffuse = (n.x * l.x + n.y * l.y + n.z * l.z).max(0.0);
    
    // Luz especular (Blinn-Phong)
    let h = Vector3::new(
        (l.x + v.x) * 0.5,
        (l.y + v.y) * 0.5,
        (l.z + v.z) * 0.5
    );
    let mut h_norm = h;
    h_norm.normalize();
    let specular = (n.x * h_norm.x + n.y * h_norm.y + n.z * h_norm.z).max(0.0).powf(32.0);
    
    (diffuse, specular)
}

// ============================================================================
// ROTACIÓN DEL PLANETA
// ============================================================================

fn rotate_position(pos: &Vector3, time: f32, speed: f32) -> Vector3 {
    let angle = time * speed;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    
    Vector3::new(
        pos.x * cos_a - pos.z * sin_a,
        pos.y,
        pos.x * sin_a + pos.z * cos_a
    )
}

// ============================================================================
// SHADER 1: PLANETA ROCOSO (Tipo Marte/Luna)
// ============================================================================
// Capas: Base terrain, cráteres, elevación, iluminación

fn rocky_planet_shader(pos: &Vector3, time: f32, normal: &Vector3) -> Vector3 {
    let rotated_pos = rotate_position(pos, time, 0.2);
    
    // CAPA 1: Terreno base con ruido fractal
    let base_noise = fbm(&rotated_pos, 5);
    
    // CAPA 2: Cráteres usando turbulencia
    let crater_scale = 8.0;
    let crater_noise = turbulence(&Vector3::new(
        rotated_pos.x * crater_scale,
        rotated_pos.y * crater_scale,
        rotated_pos.z * crater_scale
    ), 3);
    
    // CAPA 3: Elevación para montañas
    let mountain_scale = 3.0;
    let mountain_noise = fbm(&Vector3::new(
        rotated_pos.x * mountain_scale,
        rotated_pos.y * mountain_scale,
        rotated_pos.z * mountain_scale
    ), 4);
    
    // CAPA 4: Detalle fino
    let detail_noise = noise3d(&Vector3::new(
        rotated_pos.x * 12.0,
        rotated_pos.y * 12.0,
        rotated_pos.z * 12.0
    ));
    
    // Paleta de colores rocosos
    let deep_color = Vector3::new(0.3, 0.15, 0.1);  // Marrón oscuro
    let mid_color = Vector3::new(0.5, 0.3, 0.2);    // Marrón rojizo
    let high_color = Vector3::new(0.6, 0.45, 0.3);  // Arena
    let peak_color = Vector3::new(0.7, 0.6, 0.5);   // Gris claro
    
    // Combinar capas
    let elevation = (base_noise + mountain_noise) * 0.5;
    let crater_factor = (crater_noise - 0.6).max(0.0) * 2.0;
    
    let mut color = if elevation > 0.7 {
        lerp_color(&high_color, &peak_color, (elevation - 0.7) * 3.33)
    } else if elevation > 0.4 {
        lerp_color(&mid_color, &high_color, (elevation - 0.4) * 3.33)
    } else {
        lerp_color(&deep_color, &mid_color, elevation * 2.5)
    };
    
    // Aplicar cráteres (oscurecer)
    color = lerp_color(&color, &Vector3::new(0.2, 0.1, 0.05), crater_factor * 0.5);
    
    // Añadir detalle
    color = color * (0.9 + detail_noise * 0.2);
    
    // Iluminación
    let light_dir = Vector3::new(1.0, 0.5, 1.0);
    let view_dir = Vector3::new(0.0, 0.0, 1.0);
    let (diffuse, specular) = calculate_lighting(normal, &light_dir, &view_dir);
    
    let ambient = 0.15;
    color * (ambient + diffuse * 0.8) + Vector3::new(specular * 0.1, specular * 0.1, specular * 0.1)
}

// ============================================================================
// SHADER 2: GIGANTE GASEOSO (Tipo Júpiter)
// ============================================================================
// Capas: Bandas horizontales, turbulencia, tormentas, nubes

fn gas_giant_shader(pos: &Vector3, time: f32, normal: &Vector3) -> Vector3 {
    let rotated_pos = rotate_position(pos, time, 0.8);
    
    // Coordenadas esféricas para bandas
    let lat = rotated_pos.y;
    let lon = rotated_pos.x.atan2(rotated_pos.z);
    
    // CAPA 1: Bandas horizontales principales
    let band_freq = 8.0;
    let band_pattern = (lat * band_freq + time * 0.3).sin();
    
    // CAPA 2: Turbulencia atmosférica
    let turb_scale = 4.0;
    let turbulence_val = fbm(&Vector3::new(
        lon * turb_scale,
        lat * turb_scale * 0.5,
        time * 0.1
    ), 4);
    
    // CAPA 3: Gran Mancha Roja (tormenta)
    let storm_center = Vector3::new(0.3, -0.2, 0.0);
    let dist_to_storm = ((rotated_pos.x - storm_center.x).powi(2) + 
                         (rotated_pos.y - storm_center.y).powi(2) + 
                         (rotated_pos.z - storm_center.z).powi(2)).sqrt();
    let storm_factor = (1.0 - (dist_to_storm / 0.4).min(1.0)).max(0.0);
    let storm_swirl = (lon * 6.0 + turbulence_val * 3.0 + time).sin() * storm_factor;
    
    // CAPA 4: Nubes de alta altitud
    let cloud_noise = noise3d(&Vector3::new(
        lon * 16.0,
        lat * 12.0,
        time * 0.05
    ));
    
    // Paleta de colores
    let base_cream = Vector3::new(0.9, 0.85, 0.7);
    let dark_band = Vector3::new(0.6, 0.45, 0.3);
    let orange_band = Vector3::new(0.9, 0.6, 0.3);
    let storm_red = Vector3::new(0.8, 0.3, 0.2);
    let white_cloud = Vector3::new(0.95, 0.95, 0.95);
    
    // Mezclar bandas
    let band_mix = (band_pattern + turbulence_val * 0.5 + 1.0) * 0.5;
    let mut color = if band_mix > 0.65 {
        lerp_color(&base_cream, &orange_band, (band_mix - 0.65) * 2.86)
    } else if band_mix > 0.35 {
        base_cream
    } else {
        lerp_color(&dark_band, &base_cream, band_mix * 2.86)
    };
    
    // Aplicar tormenta
    color = lerp_color(&color, &storm_red, storm_factor * (0.6 + storm_swirl * 0.2));
    
    // Añadir nubes
    let cloud_factor = (cloud_noise * 0.5 + 0.5).powf(2.0) * 0.4;
    color = lerp_color(&color, &white_cloud, cloud_factor);
    
    // Iluminación suave (atmósfera difunde la luz)
    let light_dir = Vector3::new(1.0, 0.3, 1.0);
    let view_dir = Vector3::new(0.0, 0.0, 1.0);
    let (diffuse, _) = calculate_lighting(normal, &light_dir, &view_dir);
    
    let ambient = 0.3;
    color * (ambient + diffuse * 0.7)
}

// ============================================================================
// SHADER 3: PLANETA OCÉANO (Tipo Tierra acuática)
// ============================================================================
// Capas: Océanos profundos, continentes, nubes, casquetes polares

fn ocean_planet_shader(pos: &Vector3, time: f32, normal: &Vector3) -> Vector3 {
    let rotated_pos = rotate_position(pos, time, 0.4);
    
    let lat = rotated_pos.y;
    let lon = rotated_pos.x.atan2(rotated_pos.z);
    
    // CAPA 1: Terreno base (tierra vs agua)
    let terrain_noise = fbm(&rotated_pos, 4);
    let is_land = terrain_noise > 0.35;
    
    // CAPA 2: Variación oceánica
    let ocean_depth = fbm(&Vector3::new(
        rotated_pos.x * 4.0,
        rotated_pos.y * 4.0,
        rotated_pos.z * 4.0 + time * 0.1
    ), 3);
    
    // CAPA 3: Vegetación en tierra
    let vegetation = fbm(&Vector3::new(
        rotated_pos.x * 6.0,
        rotated_pos.y * 6.0,
        rotated_pos.z * 6.0
    ), 3);
    
    // CAPA 4: Nubes dinámicas
    let cloud_coverage = fbm(&Vector3::new(
        lon * 8.0,
        lat * 6.0 + time * 0.05,
        time * 0.02
    ), 4);
    
    // Colores
    let deep_ocean = Vector3::new(0.05, 0.15, 0.4);
    let shallow_ocean = Vector3::new(0.1, 0.4, 0.7);
    let beach = Vector3::new(0.8, 0.75, 0.6);
    let grass = Vector3::new(0.2, 0.6, 0.2);
    let forest = Vector3::new(0.1, 0.4, 0.15);
    let ice = Vector3::new(0.9, 0.95, 1.0);
    let cloud_white = Vector3::new(1.0, 1.0, 1.0);
    
    let mut color = if is_land {
        if terrain_noise > 0.5 {
            // Tierra alta
            if vegetation > 0.4 {
                forest
            } else {
                grass
            }
        } else {
            // Playa
            beach
        }
    } else {
        // Océano
        if ocean_depth > 0.4 {
            shallow_ocean
        } else {
            deep_ocean
        }
    };
    
    // Casquetes polares
    let ice_threshold = 0.65;
    if lat.abs() > ice_threshold {
        let ice_mix = ((lat.abs() - ice_threshold) / (1.0 - ice_threshold)).min(1.0);
        color = lerp_color(&color, &ice, ice_mix);
    }
    
    // Aplicar nubes
    let cloud_alpha = ((cloud_coverage - 0.4).max(0.0) * 2.0).min(0.7);
    color = lerp_color(&color, &cloud_white, cloud_alpha);
    
    // Iluminación
    let light_dir = Vector3::new(1.0, 0.5, 0.8);
    let view_dir = Vector3::new(0.0, 0.0, 1.0);
    let (diffuse, specular) = calculate_lighting(normal, &light_dir, &view_dir);
    
    // Especular más fuerte en océanos
    let spec_strength = if !is_land { 0.4 } else { 0.05 };
    
    let ambient = 0.2;
    color * (ambient + diffuse * 0.75) + Vector3::new(specular * spec_strength, specular * spec_strength, specular * spec_strength)
}

// ============================================================================
// SHADER 4: PLANETA VOLCÁNICO (Extra - Ciencia Ficción)
// ============================================================================
// Capas: Lava activa, corteza enfriada, emisión de luz, erupciones

fn volcanic_planet_shader(pos: &Vector3, time: f32, normal: &Vector3) -> Vector3 {
    let rotated_pos = rotate_position(pos, time, 0.15);
    
    // CAPA 1: Red de lava activa
    let lava_veins = turbulence(&Vector3::new(
        rotated_pos.x * 6.0,
        rotated_pos.y * 6.0,
        rotated_pos.z * 6.0 + time * 0.5
    ), 4);
    
    // CAPA 2: Pulso de actividad volcánica
    let pulse = (time * 2.0).sin() * 0.5 + 0.5;
    let activity = lava_veins * pulse;
    
    // CAPA 3: Erupciones localizadas
    let eruption_scale = 3.0;
    let eruption_noise = noise3d(&Vector3::new(
        rotated_pos.x * eruption_scale,
        rotated_pos.y * eruption_scale + time * 3.0,
        rotated_pos.z * eruption_scale
    ));
    
    // CAPA 4: Corteza agrietada
    let cracks = turbulence(&Vector3::new(
        rotated_pos.x * 10.0,
        rotated_pos.y * 10.0,
        rotated_pos.z * 10.0
    ), 2);
    
    // Colores
    let black_rock = Vector3::new(0.1, 0.05, 0.05);
    let cooling_lava = Vector3::new(0.4, 0.1, 0.05);
    let hot_lava = Vector3::new(0.9, 0.3, 0.1);
    let white_hot = Vector3::new(1.0, 0.9, 0.6);
    
    let mut color = if activity > 0.75 {
        // Lava muy activa
        lerp_color(&hot_lava, &white_hot, (activity - 0.75) * 4.0)
    } else if activity > 0.5 {
        // Lava caliente
        lerp_color(&cooling_lava, &hot_lava, (activity - 0.5) * 4.0)
    } else if activity > 0.3 {
        // Lava enfriándose
        lerp_color(&black_rock, &cooling_lava, (activity - 0.3) * 5.0)
    } else {
        // Roca negra
        black_rock
    };
    
    // Añadir grietas iluminadas
    if cracks > 0.7 {
        let crack_glow = (cracks - 0.7) * 3.33;
        color = lerp_color(&color, &Vector3::new(0.8, 0.2, 0.05), crack_glow * 0.5);
    }
    
    // Erupciones brillantes
    if eruption_noise > 0.8 {
        let erupt_intensity = (eruption_noise - 0.8) * 5.0;
        color = lerp_color(&color, &white_hot, erupt_intensity * pulse);
    }
    
    // Iluminación + auto-iluminación
    let light_dir = Vector3::new(1.0, 0.5, 1.0);
    let view_dir = Vector3::new(0.0, 0.0, 1.0);
    let (diffuse, _) = calculate_lighting(normal, &light_dir, &view_dir);
    
    let self_illum = activity * 0.5; // La lava emite luz
    let ambient = 0.1;
    
    color * (ambient + diffuse * 0.4 + self_illum)
}

// ============================================================================
// SHADER 5: PLANETA CRISTALINO (Extra - Ciencia Ficción)
// ============================================================================
// Capas: Estructura cristalina, reflexiones, colores prismáticos, brillo

fn crystal_planet_shader(pos: &Vector3, time: f32, normal: &Vector3) -> Vector3 {
    let rotated_pos = rotate_position(pos, time, 0.6);
    
    // CAPA 1: Estructura de cristales
    let crystal_scale = 6.0;
    let crystal_pattern = fbm(&Vector3::new(
        rotated_pos.x * crystal_scale,
        rotated_pos.y * crystal_scale,
        rotated_pos.z * crystal_scale
    ), 3);
    
    // CAPA 2: Colores prismáticos (iridiscencia)
    let hue_shift = (crystal_pattern * 10.0 + time * 0.5).sin() * 0.5 + 0.5;
    
    // CAPA 3: Vetas internas
    let internal_structure = fbm(&Vector3::new(
        rotated_pos.x * 4.0,
        rotated_pos.y * 4.0,
        rotated_pos.z * 4.0
    ), 3);
    
    // CAPA 4: Pulso de energía
    let energy_pulse = ((time * 1.5).sin() * 0.5 + 0.5) * 0.3;
    
    // Colores base del cristal
    let crystal_blue = Vector3::new(0.3, 0.6, 1.0);
    let crystal_purple = Vector3::new(0.7, 0.3, 1.0);
    let crystal_cyan = Vector3::new(0.2, 0.9, 0.9);
    let crystal_white = Vector3::new(0.95, 0.95, 1.0);
    
    // Color base según estructura
    let mut color = if hue_shift > 0.66 {
        lerp_color(&crystal_blue, &crystal_cyan, (hue_shift - 0.66) * 3.0)
    } else if hue_shift > 0.33 {
        lerp_color(&crystal_purple, &crystal_blue, (hue_shift - 0.33) * 3.0)
    } else {
        lerp_color(&crystal_cyan, &crystal_purple, hue_shift * 3.0)
    };
    
    // Añadir vetas brillantes
    if internal_structure > 0.6 {
        color = lerp_color(&color, &crystal_white, (internal_structure - 0.6) * 2.5);
    }
    
    // Iluminación especular fuerte (cristales reflejan mucho)
    let light_dir = Vector3::new(1.0, 0.5, 1.0);
    let view_dir = Vector3::new(0.0, 0.0, 1.0);
    let (diffuse, specular) = calculate_lighting(normal, &light_dir, &view_dir);
    
    let ambient = 0.3;
    color * (ambient + diffuse * 0.5) + 
    Vector3::new(specular * 0.8, specular * 0.8, specular * 0.8) +
    color * energy_pulse
}

// ============================================================================
// FUNCIONES DE RENDERIZADO ESPECIALES
// ============================================================================

pub fn render_rings(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], light: &Light) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    let mut ring_uniforms = uniforms.clone();
    ring_uniforms.render_type = 1;

    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, &ring_uniforms);
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
        fragments.extend(triangle::triangle(&tri[0], &tri[1], &tri[2], light));
    }

    // Fragment shader para anillos
    for fragment in fragments {
        let radius = (fragment.world_position.x.powi(2) + fragment.world_position.z.powi(2)).sqrt();
        
        // Bandas de colores en los anillos
        let band_pattern = (radius * 15.0).sin() * 0.5 + 0.5;
        let ring_color1 = Vector3::new(0.8, 0.7, 0.6);
        let ring_color2 = Vector3::new(0.6, 0.5, 0.4);
        let gap_color = Vector3::new(0.3, 0.2, 0.15);
        
        // Crear gaps (huecos) en los anillos
        let gap = ((radius * 20.0).sin() * 0.5 + 0.5) < 0.2;
        
        let color = if gap {
            gap_color * 0.5
        } else {
            lerp_color(&ring_color1, &ring_color2, band_pattern)
        };
        
        // Iluminación simple
        let ring_normal = Vector3::new(0.0, 1.0, 0.0);
        let light_dir = Vector3::new(1.0, 1.0, 1.0);
        let view_dir = Vector3::new(0.0, 0.0, 1.0);
        let (diffuse, _) = calculate_lighting(&ring_normal, &light_dir, &view_dir);
        
        let final_color = color * (0.3 + diffuse * 0.7);
        
        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            final_color,
            fragment.depth,
        );
    }
}

pub fn render_moon(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], light: &Light) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    let mut moon_uniforms = uniforms.clone();
    moon_uniforms.render_type = 2;

    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, &moon_uniforms);
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
        fragments.extend(triangle::triangle(&tri[0], &tri[1], &tri[2], light));
    }

    // Fragment shader para luna
    for fragment in fragments {
        // Superficie lunar con cráteres
        let crater_noise = turbulence(&Vector3::new(
            fragment.world_position.x * 8.0,
            fragment.world_position.y * 8.0,
            fragment.world_position.z * 8.0
        ), 3);
        
        let base_color = Vector3::new(0.6, 0.6, 0.6);
        let crater_color = Vector3::new(0.4, 0.4, 0.4);
        
        let color = if crater_noise > 0.7 {
            lerp_color(&base_color, &crater_color, (crater_noise - 0.7) * 3.33)
        } else {
            base_color
        };
        
        // Iluminación
        let moon_normal = fragment.world_position;
        let light_dir = Vector3::new(1.0, 1.0, 1.0);
        let view_dir = Vector3::new(0.0, 0.0, 1.0);
        let (diffuse, _) = calculate_lighting(&moon_normal, &light_dir, &view_dir);
        
        let final_color = color * (0.1 + diffuse * 0.9);
        
        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            final_color,
            fragment.depth,
        );
    }
}

// ============================================================================
// SHADER FOR THE SHIP
// ============================================================================

fn ship_shader(pos: &Vector3, _time: f32, normal: &Vector3) -> Vector3 {
    let light_dir = Vector3::new(1.0, 1.0, 1.0);
    let view_dir = Vector3::new(0.0, 0.0, 1.0); // Assuming camera is at origin looking down Z
    let (diffuse, specular) = calculate_lighting(normal, &light_dir, &view_dir);

    let base_color;

    // Create a stripe pattern based on the y-coordinate
    if (pos.y * 10.0).sin() > 0.0 {
        base_color = Vector3::new(0.8, 0.8, 0.8); // White stripe
    } else {
        base_color = Vector3::new(0.2, 0.2, 0.8); // Blue stripe
    }

    let ambient = 0.2;
    base_color * (ambient + diffuse) + Vector3::new(specular, specular, specular)
}


// ============================================================================
// FRAGMENT SHADER PRINCIPAL
// ============================================================================

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;
    let planet_type = uniforms.planet_type;
    
    let normal = fragment.normal;

    let color = match planet_type {
        0 => rocky_planet_shader(&pos, time, &normal),
        1 => gas_giant_shader(&pos, time, &normal),
        2 => ocean_planet_shader(&pos, time, &normal),
        3 => volcanic_planet_shader(&pos, time, &normal),
        4 => crystal_planet_shader(&pos, time, &normal),
        10 => ship_shader(&pos, time, &normal),
        _ => Vector3::new(0.5, 0.5, 0.5),
    };

    // Clamp valores entre 0 y 1
    Vector3::new(
        color.x.max(0.0).min(1.0),
        color.y.max(0.0).min(1.0),
        color.z.max(0.0).min(1.0),
    )
}

pub fn set_planet_type(_planet_type: i32) {
    // Función legacy - el tipo se pasa en uniforms
}
