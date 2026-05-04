//! # RenderCamera
//!
//! A simple object representing the client's point of view of the scene.

use glam::{Mat3, Mat4, Quat, UVec2, Vec3, Vec4};

/// See the module-level documentation.
pub struct Camera {
    /// Where the camera is in space.
    pub position: Vec3,
    /// The orientation of the camera: where it's looking and how it's rotated.
    pub orientation: Quat,
    /// The manner through which the 3D metrics of the scene is transformed into 2D.
    pub projection: CameraProjection,
}

#[derive(Clone, Copy)]
pub enum CameraProjection {
    /// Things which are further away look smaller.
    Perspective {
        /// Vertical field of view (in radians).
        vertical_fov_radians: f32,
        z_near_clipping_plane: f32,
        /// The far clipping plane is a cutoff for rendering, and the "1.0" for depth calculations.
        z_far_clipping_plane: f32,
    },
    /// Each 3D direction (X, Y, Z) is associated with a 2D vector.
    Axonometric {
        /// Scale of the camera. If this is `2`, an object of `1 metre` of length will fit twice on screen.
        scale: f32,
        /// For a top-down RPG perspective you'd do:
        ///
        /// ```
        /// x = Vec2(1.0, 0.0);   // to the right
        /// y = Vec2(0.0, -1.0);  // down
        /// z = Vec2(0.0, 1.0);   // up
        /// ```
        ///
        /// Note that `basis` is a `Mat3`, which allows specifying a "Z" value
        /// for your basis vectors. You might want to preserve a "Z" for normal/depth calculations.
        basis: Mat3,
        z_near_clipping_plane: f32,
        /// The far clipping plane is a cutoff for rendering, and the "1.0" for depth calculations.
        z_far_clipping_plane: f32,
    },
}

impl Camera {
    pub fn new(position: Vec3, orientation: Quat, projection: CameraProjection) -> Self {
        Camera {
            position,
            orientation,
            projection,
        }
    }

    /// Makes this camera look to a certain point.
    ///
    /// Because a camera looking at a point still has one degree of freedom (the roll),
    /// this function requires `up`, a normalized vector that further constrains the camera.
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        self.orientation = Quat::look_at_lh(self.position, target, up)
    }

    /// Returns the view matrix of this camera, used for rendering.
    pub fn view_matrix(&self, screen_size: UVec2) -> Mat4 {
        let aspect_ratio = screen_size.x as f32 / screen_size.y as f32;
        let transform = Mat4::from_rotation_translation(self.orientation, self.position);

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
                ) * transform.inverse()
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
                reshape * basis_mat * transform.inverse()
            }
        }
    }
}
