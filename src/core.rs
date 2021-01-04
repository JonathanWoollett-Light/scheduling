pub trait Distance {
    fn distance(&self, other: &Self) -> f32;
}

#[derive(Debug, Copy, Clone)]
pub struct Agent<T: Distance + Clone> {
    pub state: T,
}

#[derive(Debug)]
pub struct Task<T: Distance> {
    pub id: usize,
    pub from: T,
    pub to: T,
}
