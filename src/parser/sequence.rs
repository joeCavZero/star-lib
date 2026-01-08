use crate::core::*;

#[derive(Debug, Clone)]
pub enum StarSequence {
    Zero,
    One(StarPositionedToken),
    Two(StarPositionedToken, StarPositionedToken),
    Three(StarPositionedToken, StarPositionedToken, StarPositionedToken),
}