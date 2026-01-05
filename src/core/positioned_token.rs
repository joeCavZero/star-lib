use crate::core::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarPositionedToken {
    pub token: StarToken,
    pub position: StarPosition,
}