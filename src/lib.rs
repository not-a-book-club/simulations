#![cfg_attr(not(feature = "std"), no_std)]
#![allow(
    clippy::identity_op,
    clippy::collapsible_if,
    clippy::collapsible_else_if
)]

extern crate alloc;

pub mod grid;
pub use grid::Grid;
pub use grid::GridNew;

mod life;
pub use life::Life;

mod elementry;
pub use elementry::Elementry;

mod bitgrid;
pub use bitgrid::BitGrid;

mod bitflipper;
pub use bitflipper::BitFlipper;

pub mod prelude {
    pub use crate::bitflipper::BitFlipper;
    pub use crate::bitgrid::BitGrid;
    pub use crate::grid::{Grid, GridNew, Index};
    pub use ultraviolet::{IVec2, IVec3};
}
