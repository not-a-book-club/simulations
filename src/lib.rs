#![cfg_attr(not(feature = "std"), no_std)]
#![allow(
    clippy::identity_op,
    clippy::collapsible_if,
    clippy::collapsible_else_if
)]

extern crate alloc;

pub mod grid;
pub use grid::Grid;

mod life;
pub use life::Life;

mod elementry;
pub use elementry::Elementry;

mod bitgrid;
pub use bitgrid::BitGrid;

mod bitflipper;
pub use bitflipper::BitFlipper;
