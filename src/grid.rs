type Index = i16;

/// An abstract 2D collection of set and unset cells.
///
/// A `Grid` has a width and height which are fixed at construction.
/// Cells in a grid can be set or unset individually (with [`set`](Grid::set)) or in bulk with [`fill()`](Grid::fill).
///
/// Many methods on `Grid` have provided implementations that are correct and good enough, but your specific grid may
/// be able to implement them smarter. For example, [`BitGrid`](crate::BitGrid) stores its cells as a bit vector,
/// contiguous in memory. As such, [`BitGird::fill`](crate::BitGrid::fill) is implemented using `fill` method on `core::slice`.
pub trait Grid: Sized {
    // Construction
    fn new(width: usize, height: usize) -> Self;
    fn new_with_fn<F>(width: usize, height: usize, mut func: F) -> Self
    where
        F: FnMut(Index, Index) -> bool,
    {
        let mut grid = Self::new(width, height);
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                grid.set(x, y, func(x, y));
            }
        }
        grid
    }

    // Checking size
    fn width(&self) -> Index;
    fn height(&self) -> Index;
    fn dims(&self) -> (Index, Index) {
        (self.width(), self.height())
    }

    // Indexed access
    #[track_caller]
    fn get(&self, x: Index, y: Index) -> bool;

    #[track_caller]
    fn set(&mut self, x: Index, y: Index, elem: bool) -> bool;

    #[track_caller]
    fn flip(&mut self, x: Index, y: Index) -> bool {
        let old = self.get(x, y);
        self.set(x, y, !old);
        old
    }

    // Misc
    fn clear(&mut self) {
        self.fill(false);
    }

    fn fill(&mut self, set: bool) {
        for y in 0..self.height() {
            for x in 0..self.width() {
                self.set(x, y, set);
            }
        }
    }
}
