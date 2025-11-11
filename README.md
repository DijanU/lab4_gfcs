# Planet Shader Renderer (Laboratorio 4)

Un renderizador 3D de planetas con diferentes tipos de superficies, anillos y luna, implementado en Rust con shaders personalizados. (fotos de los planetas estan abajo). Este branch esta dedicado para el laboratorio 4 de graficas.

## Controles de la Cámara

- **Movimiento de Cámara (Orbital):**
  - `W`/`S`: Inclinar hacia arriba/abajo
  - `A`/`D`: Rotar izquierda/derecha
  - `R`/`F`: Mover cámara hacia arriba/abajo
  - `Q`/`E`: Desplazamiento lateral izquierda/derecha
  - `Flechas`: Zoom in/out y rotación horizontal

- **Selección de Planetas:**
  - `1`: Planeta rocoso
  - `2`: Gigante gaseoso
  - `3`: Planeta con agua y tierra
  - `4`: Planeta con anillos (estilo Saturno)
  - `5`: Planeta de lava

## Estructura del Proyecto

### Uniforms

La estructura `Uniforms` contiene las matrices y parámetros necesarios para el renderizado:

```rust
pub struct Uniforms {
    pub model_matrix: Matrix,      // Matriz de modelo (transformaciones del objeto)
    pub view_matrix: Matrix,       // Matriz de vista (posición/orientación de la cámara)
    pub projection_matrix: Matrix, // Matriz de proyección (perspectiva)
    pub viewport_matrix: Matrix,   // Matriz de viewport (espacio de pantalla)
    pub time: f32,                 // Tiempo transcurrido en segundos
    pub dt: f32,                   // Delta time en segundos
    pub planet_type: i32,          // 0: rocoso, 1: gaseoso, 2: personalizado, 3: con anillos, 4: de lava, 5: sol
    pub render_type: i32,          // 0: planeta, 1: anillos, 2: luna, 3: sol
}
```

### Funciones Principales

#### `vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex`
Procesa los vértices aplicando transformaciones de modelo, vista, proyección y viewport. Para el sol (`render_type = 3`), aplica un desplazamiento a los vértices para simular una superficie hirviente.

#### `fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3`
Genera el color de cada píxel basado en el tipo de planeta y efectos de iluminación.

#### Funciones de Generación de Planetas

1. **`rocky_planet_color(pos: &Vector3, time: f32) -> Vector3`**
   - Genera un planeta rocoso con cráteres y montañas.
   - Usa ruido fractal para la textura de la superficie.
   - Incluye efectos de iluminación dinámica.

2. **`gaseous_planet_color(pos: &Vector3, time: f32) -> Vector3`**
   - Crea un gigante gaseoso con bandas de colores.
   - Incluye efectos de tormentas y nubes dinámicas.

3. **`custom_planet_color(pos: &Vector3, time: f32) -> Vector3`**
   - Genera un planeta con océanos, continentes y nubes.
   - Incluye efectos de iluminación atmosférica.

4. **`ringed_planet_color(pos: &Vector3, time: f32) -> Vector3`**
   - Crea un planeta con anillos (estilo Saturno).
   - Los anillos se renderizan como un plano con textura.

5. **`lava_planet_color(pos: &Vector3, time: f32) -> Vector3`**
   - Genera un planeta volcánico con ríos de lava.
   - Incluye efectos de iluminación y emisión de luz.

6. **`sun_shader_v2(pos: &Vector3, time: f32, normal: &Vector3) -> Vector3`**
   - Genera una estrella similar al sol con una superficie animada y turbulenta.
   - **Complejidad del Shader:** Utiliza una combinación de `fbm` (Fractal Brownian Motion) y `turbulence` para crear una superficie compleja y en constante cambio. Se superponen múltiples capas de ruido con diferentes frecuencias y amplitudes para simular la turbulencia de la superficie, puntos calientes y eyecciones de masa coronal.
   - **Animación Continua:** El shader utiliza el `time` uniforme para animar las capas de ruido, creando un efecto de superficie solar en evolución continua.
   - **Emisión Variable:** La intensidad del color y la emisión de luz se modulan en función de los valores de ruido combinados, lo que da como resultado áreas más brillantes y más oscuras que simulan picos de energía y regiones más frías.
   - **Gradiente de Color Dinámico:** El color de la estrella se controla mediante un gradiente dinámico que va del rojo oscuro al naranja, al amarillo y al blanco brillante en función de la intensidad calculada a partir del ruido. Esto simula la relación entre la temperatura y el color de la superficie de una estrella.
   - **Distorsión del Vértice:** En el `vertex_shader`, cuando `render_type` es `3`, los vértices de la esfera se desplazan hacia afuera en función de una combinación de ruido animado. Esto crea un efecto de "ebullición" o "llamarada" en la superficie del sol, dándole una apariencia más volumétrica y dinámica.

#### Funciones de Renderizado Adicionales

- **`render_rings()`**: Renderiza los anillos alrededor del planeta.
- **`render_moon()`**: Renderiza la luna que orbita alrededor del planeta.

## Cómo Ejecutar

1. Asegúrate de tener Rust instalado en tu sistema.
2. Clona el repositorio.
3. Navega al directorio del proyecto.
4. Ejecuta `cargo run`.

## Requisitos

- Rust (última versión estable)
- Cargo (gestor de paquetes de Rust)
- OpenGL 3.3 o superior

## Galería

![Planet_1 Screenshot](planet1.png)
![Planet_2 Screenshot](planet2.png)
![Planet_3 Screenshot](planet3.png)
![Planet_4 Screenshot](planet4.png)
![Planet_5 Screenshot](planet5.png)