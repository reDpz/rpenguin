use cgmath::{EuclideanSpace, Matrix3, Point1, Point2, Point3, Vector1, Vector2, Vector3, Vector4};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

const SENSITIVITY_X: f64 = 0.003;
const SENSITIVITY_Y: f64 = 0.003;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    // i think this is where the camera is
    pub eye_pos: Point3<f32>,
    pub target: Vector3<f32>,
    pub up: Vector3<f32>,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let offset = Point3::from_vec(self.target + self.eye_pos.to_vec());
        let view = cgmath::Matrix4::look_at_rh(self.eye_pos, offset, self.up);

        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // can't use matrices with bytemuck
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    forward: bool,
    backward: bool,
    strafe_left: bool,
    strafe_right: bool,

    mouse_delta: (f64, f64),
    pub enabled: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            forward: false,
            backward: false,
            strafe_left: false,
            strafe_right: false,
            mouse_delta: (0.0, 0.0),
            enabled: true,
        }
    }

    pub fn process_input(&mut self, event: &WindowEvent) -> bool {
        // self.mouse_delta = (0.0, 0.0);
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
                match keycode {
                    KeyCode::KeyE => {
                        self.forward = is_pressed;
                        true
                    }
                    KeyCode::KeyS => {
                        self.strafe_left = is_pressed;
                        true
                    }
                    KeyCode::KeyD => {
                        self.backward = is_pressed;
                        true
                    }
                    KeyCode::KeyF => {
                        self.strafe_right = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, delta: &(f64, f64)) -> bool {
        if self.enabled {
            self.mouse_delta.0 += delta.0;
            self.mouse_delta.1 += delta.1;
        }
        true
    }

    pub fn update_camera(&mut self, camera: &mut Camera, delta: f32) {
        use cgmath::InnerSpace;
        let mut forward = camera.target;
        // the direction that is forward
        let forward_norm = forward.normalize();
        // not sure what this is for?
        let mut forward_mag = forward.magnitude();

        // essentially how much we are going to move
        let speed = self.speed * delta;

        // prevents glitching when camera is close to the center of scene (why??)
        if self.forward
        /* && forward_mag > self.speed */
        {
            camera.eye_pos += forward_norm * speed;
            // camera.target += forward_norm * self.speed;
        } else if self.backward {
            camera.eye_pos -= forward_norm * speed;
            // camera.target -= forward_norm * self.speed;
        }

        // holy shit it makes sense to me, use the finger rule thingy 3blue1brown, because its
        // normalized and up has a length of 1 it will essentially have a magnitude of 1
        let right = forward_norm.cross(camera.up);

        // if our positions have changed we redo the calc to keep it a buck
        if self.forward ^ self.backward {
            forward = camera.target;
            forward_mag = forward.magnitude();
        }

        if self.strafe_right {
            // WARN: I don't understand any of this

            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.eye_pos += right * speed;
            // camera.target += right * self.speed;
        } else if self.strafe_left {
            camera.eye_pos -= right * speed;
            // camera.target -= right * self.speed;
        }

        /* print!("{}[2J", 27 as char);
        println!("target: {:?}\neye: {:?}", camera.target, camera.eye_pos); */

        // Camera rotation

        let yaw = self.mouse_delta.0 * SENSITIVITY_X;
        let pitch = self.mouse_delta.1 * SENSITIVITY_Y;

        if yaw != 0.0 || pitch != 0.0 {
            camera.target = rotate_by_rad(&camera.target, yaw as f32, pitch as f32).normalize();
        }
        // reset delta
        self.mouse_delta = (0.0, 0.0);
    }
}

fn rotate_by_rad(vec: &Vector3<f32>, yaw: f32, pitch: f32) -> Vector3<f32> {
    let cosy = yaw.cos();
    let siny = yaw.sin();

    let cosp = pitch.cos();
    let sinp = pitch.sin();

    #[rustfmt::skip]
    let rotation_y:Matrix3<f32> = cgmath::Matrix3::new(
        cosy, 0.0, siny,
        0.0,1.0,0.0,
        -siny,0.0,cosy,
    );

    #[rustfmt::skip]
    let rotation_x:Matrix3<f32> = cgmath::Matrix3::new(
        1.0, 0.0, 0.0,
        0.0, cosp, -sinp,
        0.0, sinp, cosp
    );
    rotation_y * rotation_x * vec
}
