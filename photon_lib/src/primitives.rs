use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Vector2(pub OrderedFloat<f32>, pub OrderedFloat<f32>);

impl Vector2 {
    pub fn floats(&self) -> (f32, f32) {
        (self.0 .0, self.1 .0)
    }

    pub fn x(&self) -> f32 {
        self.0 .0
    }
    pub fn y(&self) -> f32 {
        self.1 .0
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Vector3(
    pub OrderedFloat<f32>,
    pub OrderedFloat<f32>,
    pub OrderedFloat<f32>,
);

impl Vector3 {
    pub fn floats(&self) -> (f32, f32, f32) {
        (self.0 .0, self.1 .0, self.2 .0)
    }

    pub fn x(&self) -> f32 {
        self.0 .0
    }
    pub fn y(&self) -> f32 {
        self.1 .0
    }
    pub fn z(&self) -> f32 {
        self.2 .0
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Quaternion(
    pub OrderedFloat<f32>,
    pub OrderedFloat<f32>,
    pub OrderedFloat<f32>,
    pub OrderedFloat<f32>,
);

impl Quaternion {
    pub fn floats(&self) -> (f32, f32, f32, f32) {
        (self.0 .0, self.1 .0, self.2 .0, self.3 .0)
    }

    pub fn x(&self) -> f32 {
        self.0 .0
    }
    pub fn y(&self) -> f32 {
        self.1 .0
    }
    pub fn z(&self) -> f32 {
        self.2 .0
    }
    pub fn w(&self) -> f32 {
        self.3 .0
    }
}
