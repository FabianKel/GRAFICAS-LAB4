
use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;


pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    let w = transformed.w;
    let transformed_position = Vec4::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w,
        1.0
    );

    let screen_position = uniforms.viewport_matrix * transformed_position;

    let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transformed_normal
    }
}

pub fn ring_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let color_palette = [
        Vec3::new(235.0 / 255.0, 91.0 / 255.0, 96.0 / 255.0),
        Vec3::new(237.0 / 255.0, 112.0 / 255.0, 47.0 / 255.0),
        Vec3::new(234.0 / 255.0, 116.0 / 255.0, 92.0 / 255.0),
        Vec3::new(235.0 / 255.0, 91.0 / 255.0, 181.0 / 255.0),
        Vec3::new(235.0 / 255.0, 165.0 / 255.0, 91.0 / 255.0),
    ];

    let noise_scale = 1.0;
    let noise_variation = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * noise_scale,
        fragment.vertex_position.y * noise_scale,
    );

    // Calcular el ángulo y la distancia para crear franjas alrededor del aro
    let angle = fragment.vertex_position.x.atan2(fragment.vertex_position.y); // Suponiendo que el aro está en el plano XY
    let radius = fragment.vertex_position.len(); // Distancia desde el centro del torus

    // Generar patrón de bandas
    let stripe_pattern = (angle * 8.0 + noise_variation * 1.5).sin();
    let color_index = ((stripe_pattern + 1.0) / 2.0 * (color_palette.len() as f32)) as usize % color_palette.len();
    let mut gas_color = color_palette[color_index] * (1.0 + noise_variation * 0.1);

    // Ajuste de intensidad para efectos de iluminación
    gas_color = gas_color * fragment.intensity;

    Color::new(
        (gas_color.x * 255.0) as u8,
        (gas_color.y * 255.0) as u8,
        (gas_color.z * 255.0) as u8,
    )
}



pub fn rocky_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Capas de la superficiergb()
    let mountain_color = Color::new(228, 179, 85);
    let valley_color = Color::new(195, 126, 50);
    let deep_color = Color::new(92, 51, 22);

    // Configuración de posición y escala
    let position = Vec3::new(
        fragment.vertex_position.x,
        fragment.vertex_position.y,
        fragment.depth,
    );
    let base_scale = 800.0;

    // Calcular ruido de terreno para simular elevación
    let base_noise = uniforms.noise.get_noise_3d(
        position.x * base_scale, 
        position.y * base_scale, 
        position.z * base_scale,
    );

    // Generar variaciones de elevación en el terreno
    let terrain_variation = uniforms.noise.get_noise_3d(
        (position.x + 1000.0) * base_scale * 0.8, 
        (position.y + 1000.0) * base_scale * 0.8, 
        (position.z + 1000.0) * base_scale * 0.8,
    ) * 0.6;

    let combined_terrain_value = (base_noise * 0.7 + terrain_variation * 0.3).clamp(0.0, 1.0);

    // Cráteres más dispersos y profundos
    let crater_distribution = 0.3; // Frecuencia baja para mayor dispersión
    let crater_depth = 3.0;
    let crater_effect = ((position.x * crater_distribution).sin() * (position.y * crater_distribution).cos()).abs() * crater_depth;
    
    // Integración de valores de ruido y cráter para la textura de superficie
    let mut surface_value = (combined_terrain_value - crater_effect).clamp(0.0, 1.0);

    // Añadir ruido fino para detalles en la superficie
    let fine_texture = uniforms.noise.get_noise_3d(
        position.x * 2000.0,
        position.y * 2000.0,
        position.z * 2000.0,
    ) * 0.35;

    surface_value = (surface_value + fine_texture).clamp(0.0, 1.0);

    // Efecto de luz direccional y sombras profundas
    let light_angle = (position.y * 0.6 + uniforms.time as f32 * 0.001).sin() * 0.2 + 0.8;
    let directional_light = (position.x * 0.25 + uniforms.time as f32 * 0.002).cos() * 0.15 + 1.0;
    let final_light_intensity = light_angle * directional_light;

    // Selección de color en función de la altura y la luz
    let base_color = if surface_value > 0.6 {
        mountain_color.lerp(&valley_color, (surface_value - 0.6) * 1.5)
    } else {
        deep_color.lerp(&valley_color, surface_value * 1.8)
    };

    // Aplicar intensidad de luz y sombras
    let mut final_color = base_color * final_light_intensity;

    // Agregar variaciones de textura adicionales para simular desgaste
    let shadow_noise = uniforms.noise.get_noise_3d(
        position.x * 3000.0,
        position.y * 3000.0,
        position.z * 3000.0,
    ) * 0.3;
    let highlight_noise = uniforms.noise.get_noise_3d(
        position.x * 3500.0,
        position.y * 3500.0,
        position.z * 3500.0,
    ) * 0.2;

    final_color = final_color * (1.0 + shadow_noise + highlight_noise);

    // Efecto de variación de profundidad y intensidad adicional
    let depth_variation = uniforms.noise.get_noise_3d(
        position.x * 4000.0,
        position.y * 4000.0,
        position.z * 4000.0,
    ) * 0.15;
    final_color = final_color * (1.0 + depth_variation);

    final_color * fragment.intensity
}





pub fn gas_giant_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let color_palette = [
        Vec3::new(235.0 / 255.0, 91.0 / 255.0, 96.0 / 255.0),
        Vec3::new(237.0 / 255.0, 112.0 / 255.0, 47.0 / 255.0),
        Vec3::new(234.0 / 255.0, 116.0 / 255.0, 92.0 / 255.0),
        Vec3::new(235.0 / 255.0, 91.0 / 255.0, 181.0 / 255.0),
        Vec3::new(235.0 / 255.0, 165.0 / 255.0, 91.0 / 255.0),
    ];

    let noise_scale = 5.0; // Escala del ruido para las variaciones
    let noise_variation = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * noise_scale,
        fragment.vertex_position.y * noise_scale,
    );

    // Generar patrón de bandas y variaciones para simular la textura gaseosa
    let stripe_pattern = (fragment.vertex_position.y * 8.0 + noise_variation * 1.5).sin();
    let color_index = ((stripe_pattern + 1.0) / 2.0 * (color_palette.len() as f32)) as usize % color_palette.len();
    let mut gas_color = color_palette[color_index] * (1.0 + noise_variation * 0.1);

    // Ajuste de intensidad para efectos de iluminación
    gas_color = gas_color * fragment.intensity;

    Color::new(
        (gas_color.x * 255.0) as u8,
        (gas_color.y * 255.0) as u8,
        (gas_color.z * 255.0) as u8,
    )
}

pub fn gas_giant_shader2(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let color_palette = [
        Vec3::new(98.0 / 255.0, 68.0 / 255.0, 217.0 / 255.0),
        Vec3::new(159.0 / 255.0, 133.0 / 255.0, 236.0 / 255.0),
        Vec3::new(84.0 / 255.0, 64.0 / 255.0, 140.0 / 255.0),
        Vec3::new(156.0 / 255.0, 100.0 / 255.0, 140.0 / 255.0),
        Vec3::new(204.0 / 255.0, 177.0 / 255.0, 210.0 / 255.0),
        Vec3::new(61.0 / 255.0, 42.0 / 255.0, 80.0 / 255.0),
    ];

    let noise_scale = 5.0; // Escala del ruido para las variaciones
    let noise_variation = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * noise_scale,
        fragment.vertex_position.y * noise_scale,
    );

    // Generar patrón de bandas verticales y variaciones para simular la textura gaseosa
    let stripe_pattern = (fragment.vertex_position.x * 8.0 + noise_variation * 1.5).sin();
    let color_index = ((stripe_pattern + 1.0) / 2.0 * (color_palette.len() as f32)) as usize % color_palette.len();
    let mut gas_color = color_palette[color_index] * (1.0 + noise_variation * 0.1);

    // Ajuste de intensidad para efectos de iluminación
    gas_color = gas_color * fragment.intensity;

    Color::new(
        (gas_color.x * 255.0) as u8,
        (gas_color.y * 255.0) as u8,
        (gas_color.z * 255.0) as u8,
    )
}

pub fn volcanic_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores para el terreno volcánico
    let lava_color = Color::new(230, 55, 15);
    let rock_color = Color::new(80, 30, 20);
    let ash_color = Color::new(100, 40, 35);

    // Posición del fragmento y escalas de ruido
    let position = Vec3::new(
        fragment.vertex_position.x,
        fragment.vertex_position.y,
        fragment.depth,
    );
    let noise_scale = 700.0;
    let lava_scale = 150.0;
    let fine_detail_scale = 3000.0;

    // Ruido básico de terreno para simular variaciones de altura en la superficie volcánica
    let terrain_noise = uniforms.noise.get_noise_3d(
        position.x * noise_scale,
        position.y * noise_scale,
        position.z * noise_scale,
    );

    // Ruido adicional para simular lagos o grietas de lava
    let lava_noise = uniforms.noise.get_noise_3d(
        position.x * lava_scale,
        position.y * lava_scale,
        position.z * lava_scale,
    );

    // Condición para identificar áreas de lava: aquellas con ruido bajo
    let is_lava = lava_noise < -0.2;

    // Ruido fino para añadir textura en las rocas volcánicas y cenizas
    let fine_detail = uniforms.noise.get_noise_3d(
        position.x * fine_detail_scale,
        position.y * fine_detail_scale,
        position.z * fine_detail_scale,
    ) * 0.3;

    // Valor combinado de terreno, lava y detalle fino
    let combined_value = (terrain_noise + fine_detail).clamp(0.0, 1.0);

    // Aplicar un color de lava o roca en función de la posición en el terreno
    let base_color = if is_lava {
        lava_color * (1.0 + (uniforms.time as f32 * 0.02).sin().abs() * 0.5)
    } else if combined_value > 0.7 {
        rock_color.lerp(&ash_color, (combined_value - 0.7) * 1.5)
    } else {
        ash_color * combined_value
    };

    // Intensidad de luz para generar sombras y detalles
    let light_angle = (position.y * 0.5 + uniforms.time as f32 * 0.002).sin() * 0.4 + 0.6;
    let directional_light = (position.x * 0.25 + uniforms.time as f32 * 0.002).cos() * 0.15 + 1.0;
    let final_light_intensity = light_angle * directional_light;

    let mut final_color = base_color * final_light_intensity;

    let shadow_noise = uniforms.noise.get_noise_3d(
        position.x * 3500.0,
        position.y * 3500.0,
        position.z * 3500.0,
    ) * 0.2;
    let highlight_noise = uniforms.noise.get_noise_3d(
        position.x * 4000.0,
        position.y * 4000.0,
        position.z * 4000.0,
    ) * 0.2;

    final_color = final_color * (1.0 + shadow_noise + highlight_noise);

    final_color * fragment.intensity
}



pub fn icy_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores de la superficie de hielo
    let ice_color = Color::new(220, 240, 255);  
    let frost_color = Color::new(150, 200, 255); 
    let deep_ice_color = Color::new(100, 180, 240);

    // Escalas de ruido para texturizar el hielo
    let position = Vec3::new(
        fragment.vertex_position.x,
        fragment.vertex_position.y,
        fragment.depth,
    );
    let ice_noise_scale = 800.0;
    let crack_scale = 200.0;    
    let fine_scale = 3000.0;

    let base_ice_noise = uniforms.noise.get_noise_3d(
        position.x * ice_noise_scale,
        position.y * ice_noise_scale,
        position.z * ice_noise_scale,
    );

    let crack_noise = uniforms.noise.get_noise_3d(
        position.x * crack_scale,
        position.y * crack_scale,
        position.z * crack_scale,
    );

    let is_crack_or_water = crack_noise < -0.1;

    let fine_detail = uniforms.noise.get_noise_3d(
        position.x * fine_scale,
        position.y * fine_scale,
        position.z * fine_scale,
    ) * 0.2;

    let combined_value = (base_ice_noise + fine_detail).clamp(0.0, 1.0);

    let base_color = if is_crack_or_water {
        deep_ice_color * (1.0 + (uniforms.time as f32 * 0.02).sin().abs() * 0.3)
    } else if combined_value > 0.6 {
        frost_color.lerp(&ice_color, (combined_value - 0.6) * 1.5)
    } else {
        ice_color * (combined_value + 1.2)
    };



    let light_angle = (position.y * 0.5 + uniforms.time as f32 * 0.001).sin() * 0.3 + 0.7;
    let directional_light = (position.x * 0.25 + uniforms.time as f32 * 0.002).cos() * 0.15 + 1.0;

    let mut final_light_intensity = light_angle * directional_light;
    if final_light_intensity < 0.5 {
        final_light_intensity = 0.5;
    }
    

    let mut final_color = base_color * final_light_intensity;

    let shadow_noise = uniforms.noise.get_noise_3d(
        position.x * 3500.0,
        position.y * 3500.0,
        position.z * 3500.0,
    ) * 0.2;
    let highlight_noise = uniforms.noise.get_noise_3d(
        position.x * 4000.0,
        position.y * 4000.0,
        position.z * 4000.0,
    ) * 0.1;

    final_color = final_color * (1.0 + shadow_noise + highlight_noise);

    final_color * fragment.intensity
}





pub fn desert_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores para las variaciones de arena
    let dark_sand_color = Color::new(106, 63, 54);
    let medium_sand_color = Color::new(200, 167, 150);
    let light_sand_color = Color::new(244, 243, 238);
    let pale_sand_color = Color::new(248, 228, 190);
    let pink_sand_color = Color::new(238, 169, 136);

    let position = Vec3::new(
        fragment.vertex_position.x,
        fragment.vertex_position.y,
        fragment.depth,
    );
    let base_scale = 500.0;

    let dune_noise = uniforms.noise.get_noise_3d(
        position.x * base_scale,
        position.y * base_scale,
        position.z * base_scale,
    );

    let dune_variation = uniforms.noise.get_noise_3d(
        (position.x + 300.0) * base_scale * 0.5,
        (position.y + 300.0) * base_scale * 0.5,
        (position.z + 300.0) * base_scale * 0.5,
    ) * 0.6;

    let combined_dune_value = (dune_noise * 0.7 + dune_variation * 0.3).clamp(0.0, 1.0);

    let fine_sand_texture = uniforms.noise.get_noise_3d(
        position.x * 2000.0,
        position.y * 2000.0,
        position.z * 2000.0,
    ) * 0.2;

    let final_dune_value = (combined_dune_value + fine_sand_texture).clamp(2.0, 5.0);

    let light_angle = (position.y * 0.6 + uniforms.time as f32 * 0.001).sin() * 0.2 + 0.8;
    let directional_light = (position.x * 0.25 + uniforms.time as f32 * 0.002).cos() * 0.15 + 1.0;
    let final_light_intensity = light_angle * directional_light;

    let base_color = if final_dune_value > 0.8 {
        dark_sand_color.lerp(&medium_sand_color, (final_dune_value - 0.8) * 5.0)
    } else if final_dune_value > 0.6 {
        medium_sand_color.lerp(&light_sand_color, (final_dune_value - 0.6) * 5.0)
    } else if final_dune_value > 0.4 {
        light_sand_color.lerp(&pale_sand_color, (final_dune_value - 0.4) * 5.0)
    } else {
        pale_sand_color.lerp(&pink_sand_color, final_dune_value * 2.5)
    };

    let mut final_color = base_color * final_light_intensity;

    let shadow_texture = uniforms.noise.get_noise_3d(
        position.x * 3000.0,
        position.y * 3000.0,
        position.z * 3000.0,
    ) * 0.2;

    final_color = final_color * (1.0 + shadow_texture);

    let dune_depth_variation = uniforms.noise.get_noise_3d(
        position.x * 4000.0,
        position.y * 4000.0,
        position.z * 4000.0,
    ) * 0.15;
    final_color = final_color * (1.0 + dune_depth_variation);

    final_color * fragment.intensity
}


pub fn water_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let shallow_water_color = Color::new(85, 170, 255);
    let deep_water_color = Color::new(10, 50, 120);
    let ocean_floor_color = Color::new(0, 25, 80);

    let position = Vec3::new(
        fragment.vertex_position.x,
        fragment.vertex_position.y,
        fragment.depth,
    );
    let base_scale = 300.0;

    let base_noise = uniforms.noise.get_noise_3d(
        position.x * base_scale,
        position.y * base_scale,
        position.z * base_scale,
    );

    let depth_variation = uniforms.noise.get_noise_3d(
        (position.x + 500.0) * base_scale * 0.4,
        (position.y + 500.0) * base_scale * 0.4,
        (position.z + 500.0) * base_scale * 0.4,
    ) * 0.5;

    let combined_depth_value = (base_noise * 0.6 + depth_variation * 0.4).clamp(0.0, 1.0);

    let ocean_depth = 0.4;
    let ocean_depth_effect = ((position.x * ocean_depth).sin() * (position.y * ocean_depth).cos()).abs() * 2.0;

    let mut water_depth_value = (combined_depth_value - ocean_depth_effect).clamp(0.0, 1.0);

    let fine_wave_texture = uniforms.noise.get_noise_3d(
        position.x * 500.0,
        position.y * 500.0,
        position.z * 500.0,
    ) * 5.5;
    

    water_depth_value = (water_depth_value + fine_wave_texture).clamp(0.0, 1.0);

    let light_angle = (position.y * 0.6 + uniforms.time as f32 * 0.001).sin() * 0.2 + 0.8;
    let directional_light = (position.x * 0.25 + uniforms.time as f32 * 0.002).cos() * 0.15 + 1.0;
    let final_light_intensity = light_angle * directional_light;

    let base_color = if water_depth_value > 0.5 {
        shallow_water_color.lerp(&deep_water_color, (water_depth_value - 0.5) * 2.0)
    } else {
        deep_water_color.lerp(&ocean_floor_color, water_depth_value * 2.0)
    };

    let mut final_color = base_color * final_light_intensity;

    let shadow_noise = uniforms.noise.get_noise_3d(
        position.x * 2500.0,
        position.y * 2500.0,
        position.z * 2500.0,
    ) * 0.25;
    let highlight_noise = uniforms.noise.get_noise_3d(
        position.x * 3500.0,
        position.y * 3500.0,
        position.z * 3500.0,
    ) * 0.15;

    final_color = final_color * (1.0 + shadow_noise + highlight_noise);

    let extra_depth_variation = uniforms.noise.get_noise_3d(
        position.x * 5000.0,
        position.y * 5000.0,
        position.z * 5000.0,
    ) * 0.1;
    final_color = final_color * (1.0 + extra_depth_variation);

    final_color * fragment.intensity
}



pub fn moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let color1 = Color::new(247, 225, 195);
    let color2 = Color::new(248, 228, 201);
    let color3 = Color::new(249, 231, 207);
    let color4 = Color::new(249, 234, 213);
    let color5 = Color::new(250, 237, 219);

    let position = Vec3::new(
        fragment.vertex_position.x,
        fragment.vertex_position.y,
        fragment.depth,
    );
    let noise_scale = 600.0;
    let crater_scale = 150.0;
    let fine_scale = 1200.0;

    let base_noise = uniforms.noise.get_noise_3d(
        position.x * noise_scale,
        position.y * noise_scale,
        position.z * noise_scale,
    );

    let crater_noise = uniforms.noise.get_noise_3d(
        position.x * crater_scale,
        position.y * crater_scale,
        position.z * crater_scale,
    );

    let is_crater = crater_noise < -0.1;

    let fine_detail = uniforms.noise.get_noise_3d(
        position.x * fine_scale,
        position.y * fine_scale,
        position.z * fine_scale,
    ) * 0.3;

    let combined_value = (base_noise + fine_detail).clamp(1.0, 2.0);

    let base_color = if is_crater {
        color3
    } else {
        let color_choice = if combined_value < 0.2 {
            color1
        } else if combined_value < 0.4 {
            color2
        } else if combined_value < 0.6 {
            color3
        } else if combined_value < 0.8 {
            color4
        } else {
            color5
        };
        color_choice * combined_value
    };

    let light_angle = (position.y * 0.3 + uniforms.time as f32 * 0.002).sin() * 0.5 + 0.5;
    let directional_light = (position.x * 0.1 + uniforms.time as f32 * 0.003).cos() * 0.2 + 0.8;
    let final_light_intensity = light_angle * directional_light;

    let mut final_color = base_color * final_light_intensity;

    let shadow_noise = uniforms.noise.get_noise_3d(
        position.x * 2000.0,
        position.y * 2000.0,
        position.z * 2000.0,
    ) * 0.1;
    let highlight_noise = uniforms.noise.get_noise_3d(
        position.x * 2500.0,
        position.y * 2500.0,
        position.z * 2500.0,
    ) * 0.05;

    final_color = final_color * (1.0 + shadow_noise + highlight_noise);

    final_color * fragment.intensity
}
