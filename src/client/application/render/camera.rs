//! # RenderCamera
//!
//! A simple object representing the client's point of view of the scene.

use glam::{Mat3, Mat4, UVec2, Vec3, Vec4};

/// See the module-level documentation.
pub struct Camera {
    pub transform: Mat4,
    pub projection: CameraProjection,
}

pub enum CameraProjection {
    Perspective {
        vertical_fov_radians: f32,
        z_near_clipping_plane: f32,
        z_far_clipping_plane: f32,
    },
    Axonometric {
        scale: f32,
        basis: Mat3,
        z_near_clipping_plane: f32,
        z_far_clipping_plane: f32,
    },
}

impl Camera {
    pub fn make_look_at_matrix(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        let z_axis = (target - eye).normalize();
        let x_axis = up.cross(z_axis).normalize();
        let y_axis = z_axis.cross(x_axis);

        Mat4::from_cols(
            x_axis.extend(0.0),
            y_axis.extend(0.0),
            z_axis.extend(0.0),
            eye.extend(1.0),
        )
    }

    pub fn view_matrix(&self, screen_size: UVec2) -> Mat4 {
        let aspect_ratio = screen_size.x as f32 / screen_size.y as f32;

        match self.projection {
            CameraProjection::Perspective {
                vertical_fov_radians,
                z_near_clipping_plane,
                z_far_clipping_plane,
            } => {
                Mat4::perspective_lh(
                    vertical_fov_radians,
                    aspect_ratio,
                    z_near_clipping_plane,
                    z_far_clipping_plane,
                ) * self.transform.inverse()
            }
            CameraProjection::Axonometric {
                scale,
                basis,
                z_near_clipping_plane,
                z_far_clipping_plane,
            } => {
                let zoom = 1.0 / scale;
                let inv_aspect = 1.0 / aspect_ratio;

                let z_range = z_far_clipping_plane - z_near_clipping_plane;

                let basis_mat = Mat4::from_mat3(basis);

                let reshape = Mat4::from_cols(
                    Vec4::new(zoom * inv_aspect, 0.0, 0.0, 0.0),
                    Vec4::new(0.0, zoom, 0.0, 0.0),
                    Vec4::new(0.0, 0.0, 1.0 / z_range, 0.0),
                    Vec4::new(0.0, 0.0, -z_near_clipping_plane / z_range, 1.0),
                );

                reshape * basis_mat * self.transform.inverse()
            }
        }
    }
}
