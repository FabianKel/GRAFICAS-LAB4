use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use std::time::{Duration, Instant};
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;

use framebuffer::Framebuffer;
use crate::fragment::Fragment;
use crate::color::Color;
use vertex::Vertex;
use obj::Obj;
use triangle::triangle;
use camera::Camera;
use shaders::{ring_shader, rocky_planet_shader, gas_giant_shader, gas_giant_shader2, volcanic_planet_shader, icy_planet_shader, desert_planet_shader, water_planet_shader, moon_shader, vertex_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType};

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: f32,
    noise: FastNoiseLite,
}

fn create_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    // Matrices de rotación y transformación combinadas.
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();
    

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );
    
    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );
    
    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;
    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );
    transform_matrix * rotation_matrix
}

fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    perspective(fov, aspect_ratio, 0.1, 1000.0)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);
    let start_time = Instant::now();

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new("Laboratorio 4 - GPC", window_width, window_height, WindowOptions::default())
        .unwrap();

    window.set_position(500, 500);
    framebuffer.set_background_color(0x333355);

    let translation = Vec3::new(0.0, 0.0, 0.0);
    let rotation = Vec3::new(0.0, 0.0, 0.0);
    let scale = 1.0f32;

    let mut camera = Camera::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
    let obj = Obj::load("assets/models/sphere2.obj").expect("Error al cargar el modelo");
    let ring_obj = Obj::load("assets/models/ring1.obj").expect("Error al cargar el modelo del aro");
    let ring_vertex_array = ring_obj.get_vertex_array();
    let vertex_array = obj.get_vertex_array();

    // Define `current_shader` como un puntero de función.
    let mut current_shader: fn(&Fragment, &Uniforms) -> Color = rocky_planet_shader;
    let mut show_ring = false;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        handle_input(&window, &mut camera, &mut current_shader, &mut show_ring);
        framebuffer.clear();

        let time_elapsed = start_time.elapsed().as_secs_f32();
        let model_matrix = create_model_matrix(translation, scale, rotation);
        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
        let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
        let uniforms = Uniforms { model_matrix, view_matrix, projection_matrix, viewport_matrix, time: time_elapsed, noise: create_noise() };

        render(&mut framebuffer, &uniforms, &vertex_array, &ring_vertex_array, current_shader, show_ring);

        window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();
        std::thread::sleep(frame_delay);
    }
}

fn handle_input(window: &Window, camera: &mut Camera, current_shader: &mut fn(&Fragment, &Uniforms) -> Color,  show_ring: &mut bool) {
    let movement_speed = 1.0;
    let rotation_speed = PI / 50.0;
    let zoom_speed = 0.1;

    // Cambia de shader según el número seleccionado.
    if window.is_key_down(Key::Key1) {
        *current_shader = rocky_planet_shader;
        *show_ring = false;
    }
    if window.is_key_down(Key::Key2) {
        *current_shader = gas_giant_shader;
    }
    if window.is_key_down(Key::Key3) {
        *current_shader = volcanic_planet_shader;
        *show_ring = false;

    }
    if window.is_key_down(Key::Key4) {
        *current_shader = icy_planet_shader;
        *show_ring = false;
    }
    if window.is_key_down(Key::Key5) {
        *current_shader = desert_planet_shader;
        *show_ring = false;
    }
    if window.is_key_down(Key::Key6) {
        *current_shader = water_planet_shader;
        *show_ring = false;
    }
    if window.is_key_down(Key::Key7) {
        *current_shader = moon_shader;
        *show_ring = false;
    }
    if window.is_key_down(Key::Key8) {
        *current_shader = gas_giant_shader2;
        *show_ring = true;
    }

    // Controles de movimiento y rotación de la cámara
    if window.is_key_down(Key::Left) {
        camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
        camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::W) {
        camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::S) {
        camera.orbit(0.0, rotation_speed);
    }

    // Movimiento de la cámara
    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
        movement.x -= movement_speed;
    }
    if window.is_key_down(Key::D) {
        movement.x += movement_speed;
    }
    if window.is_key_down(Key::Q) {
        movement.y += movement_speed;
    }
    if window.is_key_down(Key::E) {
        movement.y -= movement_speed;
    }
    if movement.magnitude() > 0.0 {
        camera.move_center(movement);
    }

    // Zoom de la cámara
    if window.is_key_down(Key::Up) {
        camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.zoom(-zoom_speed);
    }
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], ring_vertex_array: &[Vertex], current_shader: fn(&Fragment, &Uniforms) -> Color, show_ring: bool) {
    let transformed_vertices = vertex_array.iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect::<Vec<_>>();

    let triangles = transformed_vertices.chunks(3)
        .filter(|tri| tri.len() == 3)
        .map(|tri| [tri[0].clone(), tri[1].clone(), tri[2].clone()])
        .collect::<Vec<_>>();

    let mut fragments = Vec::new();

    // Renderiza los planetas
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Renderiza el aro
    if show_ring {
        let translation = Vec3::new(0.0, -0.4, 0.0);

        let transformed_ring_vertices = ring_vertex_array.iter()
            .map(|vertex| {
                let mut transformed_vertex = vertex.clone();
                transformed_vertex.position.x += translation.x;
                transformed_vertex.position.y += translation.y;
                transformed_vertex.position.z += translation.z;
                vertex_shader(&transformed_vertex, uniforms)
            })
            .collect::<Vec<_>>();

        let ring_triangles = transformed_ring_vertices.chunks(3)
            .filter(|tri| tri.len() == 3)
            .map(|tri| [tri[0].clone(), tri[1].clone(), tri[2].clone()])
            .collect::<Vec<_>>();

        let ring_shader_fn: fn(&Fragment, &Uniforms) -> Color = ring_shader;

        for tri in &ring_triangles {
            let ring_fragments: Vec<Fragment> = triangle(&tri[0], &tri[1], &tri[2]);
            for fragment in ring_fragments {
                let (x, y) = (fragment.position.x as usize, fragment.position.y as usize);
                if x < framebuffer.width && y < framebuffer.height {
                    let shaded_color = ring_shader_fn(&fragment, uniforms);
                    framebuffer.set_current_color(shaded_color.to_hex());
                    framebuffer.point(x, y, fragment.depth);
                }
            }
        }
    }

    for fragment in fragments {
        let (x, y) = (fragment.position.x as usize, fragment.position.y as usize);
        if x < framebuffer.width && y < framebuffer.height {
            let shaded_color = current_shader(&fragment, uniforms);
            framebuffer.set_current_color(shaded_color.to_hex());
            framebuffer.point(x, y, fragment.depth);
        }
    }
}
