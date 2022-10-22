use ordered_float::OrderedFloat;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Vector2(pub OrderedFloat<f32>, pub OrderedFloat<f32>);

impl Vector2 {
    pub fn floats(&self) -> (f32, f32) {
        (self.0 .0, self.1 .0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Vector3(
    pub OrderedFloat<f32>,
    pub OrderedFloat<f32>,
    pub OrderedFloat<f32>,
);

impl Vector3 {
    pub fn floats(&self) -> (f32, f32, f32) {
        (self.0 .0, self.1 .0, self.2 .0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
}
