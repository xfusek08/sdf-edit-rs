use std::mem::size_of;

use glam::Vec3;

/// A trait which each vertex type must implement.
pub trait Vertex: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
}

/// Simple Vertex
/// A vertex holding just a position.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVertex(pub Vec3);
impl Vertex for SimpleVertex {}

/// Color Vertex
/// A vertex type which contains position and color.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    pub position: Vec3,
    pub color: Vec3,
}
impl Vertex for ColorVertex {}
