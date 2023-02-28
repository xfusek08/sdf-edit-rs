
use glam::Vec4Swizzles;

use super::{BoundingCube, Transform, Frustum, PositionRelativeToFrustum, HalfSpace, Plane};

#[derive(Debug, Clone)]
pub struct AABB {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl AABB {
    
    pub const ZERO: Self = Self {
        min: glam::Vec3::ZERO,
        max: glam::Vec3::ZERO,
    };
    
    pub fn new(min: glam::Vec3, max: glam::Vec3) -> Self {
        Self { min, max }
    }
    
    pub fn from_bounding_cube(bounding_cube: &BoundingCube) -> Self {
        let half_size = bounding_cube.size * 0.5;
        let min = bounding_cube.pos - glam::Vec3::splat(half_size);
        let max = bounding_cube.pos + glam::Vec3::splat(half_size);
        Self { min, max }
    }
    
    pub fn bounding_cube(&self) -> BoundingCube {
        let size = (self.max - self.min).max_element();
        let pos = self.min + (self.max - self.min) * 0.5;
        BoundingCube { pos, size }
    }
    
    /// Creates 8 vertices of an aabb and rotate them, then create a new aabb from the rotated vertices
    pub fn vertices(&self) -> [glam::Vec3; 8] {
        [
            self.min,
            glam::Vec3::new(self.min.x, self.min.y, self.max.z),
            glam::Vec3::new(self.min.x, self.max.y, self.min.z),
            glam::Vec3::new(self.min.x, self.max.y, self.max.z),
            glam::Vec3::new(self.max.x, self.min.y, self.min.z),
            glam::Vec3::new(self.max.x, self.min.y, self.max.z),
            glam::Vec3::new(self.max.x, self.max.y, self.min.z),
            self.max,
        ]
    }
    
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
    
    #[inline]
    pub fn rotate(&self, rotation: &glam::Quat) -> Self {
        if rotation.is_near_identity() {
            return self.clone();
        }
        
        let mut min = glam::Vec3::splat(f32::MAX);
        let mut max = glam::Vec3::splat(f32::MIN);
        for vertex in self.vertices().iter() {
            let rotated_vertex = *rotation * *vertex;
            min = min.min(rotated_vertex);
            max = max.max(rotated_vertex);
        }
        Self { min, max }
    }
    
    #[inline]
    pub fn translate(&self, translation: &glam::Vec3) -> Self {
        Self {
            min: self.min + *translation,
            max: self.max + *translation,
        }
    }
    
    #[inline]
    pub fn scale(&self, scale: &glam::Vec3) -> Self {
        Self {
            min: self.min * *scale,
            max: self.max * *scale,
        }
    }
    
    #[inline]
    pub fn transform(&self, transform: &Transform) -> Self {
        self
            .scale(&transform.scale)
            .rotate(&transform.rotation)
            .translate(&transform.position)
    }
    
    #[inline]
    pub fn inflate(&self, amount: f32) -> Self {
        Self {
            min: self.min - glam::Vec3::splat(amount),
            max: self.max + glam::Vec3::splat(amount),
        }
    }
    
    pub fn in_frustum(&self, frustum: &Frustum) -> bool {
        
        let is_intersecting_into_positive_half_space = |plane: &Plane| {
            self.vertices().iter().any(|v| { plane.classify_point(v) == HalfSpace::Positive})
        };
        
        let is_at_least_one_vertex_inside = frustum.planes().iter().all(is_intersecting_into_positive_half_space);
        
        is_at_least_one_vertex_inside
        
        // NOTE: following code is not working right for some reason
        // // Compute if all frustum vertices are outside of the aabb according to: https://iquilezles.org/articles/frustumcorrect/
        // if frustum.vertices().iter().all(|fv| {fv.x >  self.max.x}) { return false; }
        // if frustum.vertices().iter().all(|fv| {fv.x <  self.min.x}) { return false; }
        // if frustum.vertices().iter().all(|fv| {fv.y >  self.max.y}) { return false; }
        // if frustum.vertices().iter().all(|fv| {fv.y <  self.min.y}) { return false; }
        // if frustum.vertices().iter().all(|fv| {fv.z >  self.max.z}) { return false; }
        // if frustum.vertices().iter().all(|fv| {fv.z <  self.min.z}) { return false; }
        // true
    }
}

// GPU aligned version
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AABBAligned {
    pub min: glam::Vec4,
    pub max: glam::Vec4,
}

// implement only from AABB and to AABB
impl AABBAligned {
    pub fn new(min: glam::Vec3, max: glam::Vec3) -> Self {
        Self {
            min: glam::Vec4::new(min.x, min.y, min.z, 0.0),
            max: glam::Vec4::new(max.x, max.y, max.z, 0.0),
        }
    }
    
    pub fn from_aabb(aabb: &AABB) -> Self {
        Self::new(aabb.min, aabb.max)
    }
    
    pub fn to_aabb(&self) -> AABB {
        AABB::new(self.min.xyz(), self.max.xyz())
    }
}
