use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

// Constants
impl Transform {
    pub const IDENTITY: Self = Self {
        position: glam::Vec3::ZERO,
        rotation: glam::Quat::IDENTITY,
        scale: glam::Vec3::ONE
    };
}

// Factories
impl Transform {
    pub fn from_uniform_scale(scale: f32) -> Self {
        Self {
            scale: glam::Vec3::splat(scale),
            ..Self::IDENTITY
        }
    }
    
    pub fn from_polar(radius: f32, theta: f32, phi: f32) -> Self {
        Self {
            position: glam::Vec3::new(
                radius * theta.sin() * phi.sin(),
                radius * theta.cos(),
                radius * theta.sin() * phi.cos(),
            ),
            ..Self::IDENTITY
        }
    }
    
    pub fn from_vec3(position: glam::Vec3) -> Self {
        Self {
            position,
            ..Self::IDENTITY
        }
    }
    
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_vec3(glam::Vec3::new(x, y, z))
    }
}

// Getters
impl Transform {
    pub fn as_mat(&self)   -> glam::Mat4 { glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position) }
}

// Builders
impl Transform {
    pub fn with_position(&self, position: glam::Vec3) -> Self {
        Self { position, ..*self }
    }
    pub fn with_rotation(&self, rotation: glam::Quat) -> Self {
        Self { rotation, ..*self }
    }
    pub fn with_scale(&self, scale: glam::Vec3) -> Self {
        Self { scale, ..*self }
    }
}

// Operations
impl Transform {
    #[inline]
    pub fn translate(&self, translation: glam::Vec3) -> Self {
        Self {
            position: self.position + translation,
            ..*self
        }
    }
    
    #[inline]
    pub fn rotate(&self, rotation: glam::Quat) -> Self {
        Self {
            rotation: self.rotation * rotation,
            ..*self
        }
    }
    
    #[inline]
    pub fn scale(&self, scale: glam::Vec3) -> Self {
        Self {
            scale: self.scale * scale,
            ..*self
        }
    }
    
    #[inline]
    pub fn scale_evenly(&self, scale: f32) -> Self {
        self.scale(glam::Vec3::splat(scale))
    }
    
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            position: self.position + other.position,
            rotation: self.rotation * other.rotation,
            scale: self.scale * other.scale,
        }
    }
}

// Default
impl Default for Transform {
    fn default() -> Self { Self::IDENTITY }
}
