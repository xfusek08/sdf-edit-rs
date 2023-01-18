
use super::{BoundingCube, Transform};

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
        
        // create 8 vertices of an aabb and rotate them, then create a new aabb from the rotated vertices
        let vertices = [
            self.min,
            glam::Vec3::new(self.min.x, self.min.y, self.max.z),
            glam::Vec3::new(self.min.x, self.max.y, self.min.z),
            glam::Vec3::new(self.min.x, self.max.y, self.max.z),
            glam::Vec3::new(self.max.x, self.min.y, self.min.z),
            glam::Vec3::new(self.max.x, self.min.y, self.max.z),
            glam::Vec3::new(self.max.x, self.max.y, self.min.z),
            self.max,
        ];
        let mut min = glam::Vec3::splat(f32::MAX);
        let mut max = glam::Vec3::splat(f32::MIN);
        for vertex in vertices.iter() {
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
            .rotate(&transform.rotation)
            .translate(&transform.position)
            .scale(&transform.scale)
    }
    
    #[inline]
    pub fn inflate(&self, amount: f32) -> Self {
        Self {
            min: self.min - glam::Vec3::splat(amount),
            max: self.max + glam::Vec3::splat(amount),
        }
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
        Self::new(
            glam::Vec3::new(aabb.min.x, aabb.min.y, aabb.min.z),
            glam::Vec3::new(aabb.max.x, aabb.max.y, aabb.max.z),
        )
    }
    
    pub fn to_aabb(&self) -> AABB {
        AABB::new(
            glam::Vec3::new(self.min.x, self.min.y, self.min.z),
            glam::Vec3::new(self.max.x, self.max.y, self.max.z),
        )
    }
    
    pub fn bounding_cube(&self) -> BoundingCube {
        self.to_aabb().bounding_cube()
    }
    
    pub fn from_bounding_cube(bounding_cube: &BoundingCube) -> Self {
        Self::from_aabb(&AABB::from_bounding_cube(bounding_cube))
    }
    
    pub fn add(&self, other: &Self) -> Self {
        Self::from_aabb(&self.to_aabb().add(&other.to_aabb()))
    }
    
    pub fn rotate(&mut self, rotation: &glam::Quat) {
        self.to_aabb().rotate(rotation);
    }
    
    pub fn translate(&mut self, translation: &glam::Vec3) {
        self.to_aabb().translate(translation);
    }
    
    pub fn scale(&mut self, scale: &glam::Vec3) {
        self.to_aabb().scale(scale);
    }
    
    pub fn transform(&mut self, transform: &Transform) {
        self.to_aabb().transform(transform);
    }
}
