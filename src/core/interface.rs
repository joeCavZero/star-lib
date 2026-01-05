use crate::core::StarMcallContext;

pub trait StarInterface {
    fn mcall(&mut self, s: &mut dyn StarMcallContext) -> bool;
}