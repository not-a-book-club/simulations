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
    // TODO: It'd be nice to gave Grid::new() behind Clone, so we can have &mut T types impl Grid
    // Construction
    fn new(width: usize, height: usize) -> Self;

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

#[cfg(test)]
mod tests {
    use crate::BitFlipper;

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct TestGridWithMut<'a, G: Grid> {
        grid: G,
        flipped: &'a mut Vec<(Index, Index)>,
    }

    impl<G: Grid> Grid for TestGridWithMut<'_, G> {
        fn new(width: usize, height: usize) -> Self {
            unreachable!(
                "Not expected to be called by bitflipper: new(width: {width}, height: {height})"
            );
        }

        fn width(&self) -> Index {
            self.grid.width()
        }

        fn height(&self) -> Index {
            self.grid.height()
        }

        fn get(&self, x: Index, y: Index) -> bool {
            let elem = self.grid.get(x, y);
            unreachable!("Not expected to be called by bitflipper: get(x: {x}, y: {y}) == {elem}");
        }

        fn set(&mut self, x: Index, y: Index, elem: bool) -> bool {
            unreachable!(
                "Not expected to be called by bitflipper: set(x: {x}, y: {y}, elem: {elem})"
            );
        }

        fn flip(&mut self, x: Index, y: Index) -> bool {
            dbg!((x, y));
            self.flipped.push((x, y));
            self.grid.flip(x, y)
        }
    }

    // Make sure certain types of Grid impls are possible
    #[test]
    fn check_flip_tracking_with_mut() {
        let bitgrid = crate::BitGrid::new(32, 32);
        let mut flipped = vec![];
        let mut bitflipper = BitFlipper::new_with_grid(
            TestGridWithMut {
                grid: bitgrid,
                flipped: &mut flipped,
            },
            1, // dx
            1, // dy
        );

        // Note: Must use grid()/grid_mut() because:
        //      error[E0502]: cannot borrow `flipped` as immutable because it is also borrowed as mutable
        //         --> src/grid.rs:111:14
        //          |
        //      105 |                 flipped: &mut flipped,
        //          |                          ------------ mutable borrow occurs here
        //      ...
        //      111 |         dbg!(&flipped);
        //          |              ^^^^^^^^ immutable borrow occurs here
        //      112 |
        //      113 |         bitflipper.flip_and_advance(1);
        //          |         ---------- mutable borrow later used here
        // dbg!(&flipped);

        bitflipper.flip_and_advance(1);
        assert_eq!(bitflipper.grid().flipped, &[(31, 31)]);
        bitflipper.grid_mut().flipped.clear();

        bitflipper.flip_and_advance(1);
        assert_eq!(bitflipper.grid().flipped, &[(30, 30)]);
        bitflipper.grid_mut().flipped.clear();

        bitflipper.flip_and_advance(1);
        assert_eq!(bitflipper.grid().flipped, &[(29, 29)]);
        bitflipper.grid_mut().flipped.clear();

        bitflipper.flip_and_advance(1);
        assert_eq!(bitflipper.grid().flipped, &[(28, 28)]);
        bitflipper.grid_mut().flipped.clear();
    }
}
