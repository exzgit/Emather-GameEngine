use nalgebra::Matrix4;
use crate::engine::scene::game_object::Transform;

#[allow(unused)]
pub struct Model {
    pub model_matrix: Matrix4<f32>,
}

#[allow(unused)]
impl Model {
    pub fn update(&mut self, transform: &Transform) {
        // Update model matrix based on transform
        self.model_matrix = transform.to_matrix();
    }
} 