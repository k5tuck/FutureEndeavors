//! Camera system for 2D and 3D simulations

use glam::{Mat4, Vec3};

/// 2D orthographic camera
#[derive(Debug, Clone)]
pub struct Camera2D {
    pub position: Vec3,
    pub zoom: f32,
    pub aspect_ratio: f32,
}

impl Camera2D {
    pub fn new(aspect_ratio: f32) -> Self {
        Self {
            position: Vec3::ZERO,
            zoom: 1.0,
            aspect_ratio,
        }
    }

    /// Get the view-projection matrix
    pub fn view_projection(&self) -> Mat4 {
        let half_width = self.zoom * self.aspect_ratio;
        let half_height = self.zoom;

        let projection = Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            -1.0,
            1.0,
        );

        let view = Mat4::from_translation(-self.position);

        projection * view
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }
}

/// 3D perspective camera with orbital controls
#[derive(Debug, Clone)]
pub struct Camera3D {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    // Orbital parameters
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera3D {
    pub fn new(aspect_ratio: f32) -> Self {
        let distance = 10.0;
        let yaw = 0.0f32;
        let pitch = 0.3f32;

        let position = Vec3::new(
            distance * pitch.cos() * yaw.sin(),
            distance * pitch.sin(),
            distance * pitch.cos() * yaw.cos(),
        );

        Self {
            position,
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: 45.0f32.to_radians(),
            aspect_ratio,
            near: 0.1,
            far: 1000.0,
            distance,
            yaw,
            pitch,
        }
    }

    /// Update camera position based on orbital parameters
    pub fn update_orbital(&mut self) {
        self.position = self.target + Vec3::new(
            self.distance * self.pitch.cos() * self.yaw.sin(),
            self.distance * self.pitch.sin(),
            self.distance * self.pitch.cos() * self.yaw.cos(),
        );
    }

    /// Orbit the camera around the target
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(-1.5, 1.5);
        self.update_orbital();
    }

    /// Zoom in/out
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance - delta).max(1.0);
        self.update_orbital();
    }

    /// Get the view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Get the projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }

    /// Get the combined view-projection matrix
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }
}

/// Camera uniform data for shaders
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub position: [f32; 4],
}

impl CameraUniform {
    pub fn from_camera_3d(camera: &Camera3D) -> Self {
        Self {
            view_proj: camera.view_projection().to_cols_array_2d(),
            position: [camera.position.x, camera.position.y, camera.position.z, 1.0],
        }
    }

    pub fn from_camera_2d(camera: &Camera2D) -> Self {
        Self {
            view_proj: camera.view_projection().to_cols_array_2d(),
            position: [camera.position.x, camera.position.y, camera.position.z, 1.0],
        }
    }
}
