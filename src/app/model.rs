///! Temporary model date definitions for the app.

use glam::Vec3;

use super::gpu::vertices::ColorVertex;

// pub const PENTAGON_VERTICES: &[Vertex] = &[
//     Vertex { position: Vec3::new(-0.0868241, 0.49240386, 0.0), tex_coords: Vec2::new(0.4131759, 0.00759614) },
//     Vertex { position: Vec3::new(-0.49513406, 0.06958647, 0.0), tex_coords: Vec2::new(0.0048659444, 0.43041354) },
//     Vertex { position: Vec3::new(-0.21918549, -0.44939706, 0.0), tex_coords: Vec2::new(0.28081453, 0.949397) },
//     Vertex { position: Vec3::new(0.35966998, -0.3473291, 0.0), tex_coords: Vec2::new(0.85967, 0.84732914) },
//     Vertex { position: Vec3::new(0.44147372, 0.2347359, 0.0), tex_coords: Vec2::new(0.9414737, 0.2652641) },
// ];

// pub const PENTAGON_INDICES: &[u16] = &[
//     0, 1, 4,
//     1, 2, 4,
//     2, 3, 4,
// ];

// pub struct Model {
//     pub vertices: &'static [Vertex],
//     pub indices: &'static [u16],
//     pub texture: image::DynamicImage,
// }


pub const AXIS_VERTICES: &[ColorVertex] = &[
    ColorVertex { position: Vec3::new(-1.0, 0.0, 0.0), color: Vec3::new(1.0, 0.0, 0.0) },
    ColorVertex { position: Vec3::new(1.0, 0.0, 0.0),  color: Vec3::new(1.0, 0.0, 0.0) },
    ColorVertex { position: Vec3::new(0.0, -1.0, 0.0), color: Vec3::new(0.0, 1.0, 0.0) },
    ColorVertex { position: Vec3::new(0.0, 1.0, 0.0),  color: Vec3::new(0.0, 1.0, 0.0) },
    ColorVertex { position: Vec3::new(0.0, 0.0, -1.0), color: Vec3::new(0.0, 0.0, 1.0) },
    ColorVertex { position: Vec3::new(0.0, 0.0, 1.0),  color: Vec3::new(0.0, 0.0, 1.0) },
];
