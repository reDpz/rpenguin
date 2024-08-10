use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
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
}

impl Camera2D {
    pub fn new(aspect: f32) -> Self {
        Self {
            aspect,
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::from_rotation_z(0.0),
            zoom: 1.0,
        }
    }

    #[inline]
    pub fn build_projection_matrix(&self) -> glam::Mat4 {
        // warp x to transform world space to screen space
        let scale = glam::Vec3 {
            x: self.zoom * self.aspect,
            y: self.zoom,
            z: self.zoom,
        };

        // WARN: might need to transform using OPENGL_TO_WGPU_MATRIX
        glam::Mat4::from_scale_rotation_translation(scale, self.rotation, self.position)
    }
}

pub struct CameraController2D {
    // properties
    /// units traveled per second
    pub speed: f32,
    // inputs
    pub action_set: Vec<(KeyCode, bool)>,
}

struct ActionSet;
impl ActionSet {
    const UP: usize = 0;
    const DOWN: usize = 1;
    const LEFT: usize = 2;
    const RIGHT: usize = 3;

    const CLOCKWISE: usize = 4;
    const ACLOCKWISE: usize = 5;

    const SIZE: usize = 6;
}

impl CameraController2D {
    pub fn new() -> Self {
        let mut action_set = Vec::with_capacity(ActionSet::SIZE);
        action_set[ActionSet::UP] = (KeyCode::KeyW, false);
        action_set[ActionSet::DOWN] = (KeyCode::KeyS, false);
        action_set[ActionSet::LEFT] = (KeyCode::KeyA, false);
        action_set[ActionSet::RIGHT] = (KeyCode::KeyA, false);
        action_set[ActionSet::CLOCKWISE] = (KeyCode::KeyE, false);
        action_set[ActionSet::ACLOCKWISE] = (KeyCode::KeyQ, false);

        Self {
            speed: 1.0,
            action_set,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
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
                for (action_code, mut pressed) in &self.action_set {
                    if *action_code == *keycode {
                        pressed = is_pressed;
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera2D, delta: f32) {
        let spelta = self.speed * delta;
        if self.action_set[ActionSet::RIGHT].1 {
            camera.position.x += spelta;
        }
        if self.action_set[ActionSet::LEFT].1 {
            camera.position -= spelta;
        }
    }

    pub fn dinput(&mut self, event: &DeviceEvent) {
        todo!()
    }
}
