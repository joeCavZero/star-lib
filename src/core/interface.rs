use crate::core::Star;

pub trait Interface {
    fn mcall(&mut self, s: &mut Star) -> bool;
}