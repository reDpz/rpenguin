use glam::Vec4Swizzles;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

const SENSITIVITY_X: f64 = 0.003;
const SENSITIVITY_Y: f64 = 0.003;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
]);

pub struct Camera2D {
    pub aspect: f32,
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub zoom: f32,

    pub proj: glam::Mat4,
}

impl Camera2D {
    pub fn new(aspect: f32) -> Self {
        Self {
            aspect,

            position: glam::Vec3::ZERO,
            rotation: glam::Quat::from_rotation_z(0.0),
            zoom: 1.0,
            proj: glam::Mat4::ZERO,
        }
    }

    pub fn update_projection_matrix(&mut self) {
        // WARN: might need to transform using OPENGL_TO_WGPU_MATRIX
        // TODO: Math to figure the correct way of applying zoom
        let zoomsqred = self.zoom * self.zoom;
        let left = (-zoomsqred + self.position.x);

        let right = (zoomsqred + self.position.x);

        let top = (zoomsqred + self.position.y) * self.aspect;
        let bottom = (-zoomsqred + self.position.y) * self.aspect;

        self.proj = glam::Mat4::orthographic_lh(left, right, bottom, top, 0.0, 1.0);

        // self.proj =
        //     glam::Mat4::from_scale_rotation_translation(scale, self.rotation, self.position);
    }
}

pub struct CameraController2D {
    // properties
    /// units traveled per second
    pub speed: f32,
    pub markiplier: f32,
    pub zoom_step: f32,

    // inputs
    // keyboard
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub mod_key: bool,

    // mouse
    pub scroll_delta: f32,
}

impl Default for CameraController2D {
    fn default() -> Self {
        Self {
            speed: 1.0,
            zoom_step: 0.2,
            markiplier: 4.0,
            up: false,
            down: false,
            left: false,
            right: false,
            mod_key: false,
            scroll_delta: 0.0,
        }
    }
}

impl CameraController2D {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            ..Default::default()
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, ydelta),
                ..
            } => {
                self.scroll_delta -= ydelta;
                true
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = state.is_pressed();

                // TODO: REFACTOR INPUTS
                // this should be short enough, only 6 cycles currently but this is suboptimal
                match keycode {
                    KeyCode::KeyW => {
                        self.up = is_pressed;
                        true
                    }

                    KeyCode::KeyD => {
                        self.right = is_pressed;
                        true
                    }

                    KeyCode::KeyA => {
                        self.left = is_pressed;
                        true
                    }

                    KeyCode::KeyS => {
                        self.down = is_pressed;
                        true
                    }

                    KeyCode::ShiftLeft => {
                        self.mod_key = is_pressed;
                        true
                    }

                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn process(&mut self, camera: &mut Camera2D, delta: f32) {
        let mut spelta = self.speed * delta * camera.zoom;

        if self.mod_key {
            spelta *= self.markiplier;
        }

        let mut translation = glam::Vec3::ZERO;

        if self.left {
            translation.x -= 1.0;
        }

        if self.right {
            translation.x += 1.0;
        }

        if self.down {
            translation.y -= 1.0;
        }

        if self.up {
            translation.y += 1.0;
        }

        translation = translation.normalize_or_zero() * spelta;

        // zooming in and out
        if self.scroll_delta != 0.0 {
            camera.zoom = (camera.zoom + self.scroll_delta * self.zoom_step).max(0.1);
            self.scroll_delta = 0.0;
        }

        camera.position += translation;
    }

    pub fn dinput(&mut self, event: &DeviceEvent) {
        todo!()
    }
}
