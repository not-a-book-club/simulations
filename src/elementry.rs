use crate::Grid;

#[derive(Clone)]
pub struct Elementry<G: Grid = crate::BitGrid> {
    /// Current state of the simulation
    cells: G,

    /// Scratch copy of cells used when stepping the simulation
    scratch: G,

    rule: u8,
}

/// Basic Usage
impl<G: Grid + Clone> Elementry<G> {
    /// Creates a new `Elementry` simulation with the given rule and dimensions with all cells initially **dead**.
    pub fn new(rule: u8, width: usize) -> Self {
        Self::new_with_cells(rule, G::new(width, 1))
    }

    /// Creates a new `Elementry` simulation with the given rule and existing cells.
    ///
    /// Note: `cells` must be 1 dimensional (cells.height() == 1) or this method will panic.
    pub fn new_with_cells(rule: u8, cells: G) -> Self {
        assert_eq!(
            cells.height(),
            1,
            "Elementary only operates on a 1D grid of cells"
        );
        let scratch = cells.clone();
        Self {
            cells,
            scratch,
            rule,
        }
    }
}

impl<G: Grid> Elementry<G> {
    /// The width of the simulation
    pub fn width(&self) -> i16 {
        self.cells.width()
    }

    pub fn cells(&self) -> impl Iterator<Item = bool> + '_ {
        (0..self.width()).map(|i| self.get(i))
    }

    pub fn get(&self, x: i16) -> bool {
        self.cells.get(x, 1)
    }

    pub fn set(&mut self, x: i16, is_alive: bool) {
        self.cells.set(x, 1, is_alive);
    }

    /// Steps the simulation once, returning the number of cells updated
    ///
    /// Note: If this ever returns `0`, the simulation will henceforth never change, because nothing is changing anymore.
    pub fn step(&mut self) -> u32 {
        let mut count = 0;

        // Modify scratch while we step because we must keep the immediate previous version unmodified.
        for x in 0..self.width() {
            let old = self.get(x);
            let c = ((self.get(x - 1) as u8) << 2)
                | ((self.get(x + 0) as u8) << 1)
                | ((self.get(x + 1) as u8) << 0);
            let mask = 1 << c;

            let is_alive = (self.rule & mask) != 0;
            self.scratch.set(x, 1, is_alive);

            count += (old != is_alive) as u32;
        }

        // Now that we're done stepping, we can swap our scrap and official cells.
        // Note: We expect Grids to be heap-allocated so this is quick enough.
        // TODO: We could bounce with indices instead. Might even be faster.
        core::mem::swap(&mut self.cells, &mut self.scratch);

        count
    }

    /// Marks all cells as **dead**
    pub fn clear(&mut self) {
        self.cells.fill(false);
    }

    /// Marks all cells as **alive**
    pub fn clear_alive(&mut self) {
        self.cells.fill(true);
    }
}

impl Elementry<crate::BitGrid> {
    /// Set all cells to **alive** or **dead** using the provided rng.
    pub fn clear_random(&mut self, rng: &mut impl rand::Rng) {
        let bytes: &mut [u8] = self.cells.as_mut_bytes();
        for chunk in bytes.chunks_mut(4) {
            let rand_bytes = rng.next_u32().to_le_bytes();
            chunk.copy_from_slice(&rand_bytes[..chunk.len()]);
        }
    }
}

/// `std`-only functions
#[cfg(feature = "std")]
impl Elementry {
    pub fn to_ascii(&self) -> String {
        self.cells()
            .map(|is_alive| if is_alive { 'O' } else { '.' })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn check_single_seed() {
        let mut sim = Elementry::new(30, 64);
        sim.set(32, true);

        assert_eq!(sim.to_ascii(), "................................O...............................");
        sim.step();

        assert_eq!(sim.to_ascii(), "...............................OOO..............................");
        sim.step();

        assert_eq!(sim.to_ascii(), "..............................OO..O.............................");
        sim.step();

        assert_eq!(sim.to_ascii(), ".............................OO.OOOO............................");
        sim.step();

        assert_eq!(sim.to_ascii(), "............................OO..O...O...........................");
        sim.step();

        assert_eq!(sim.to_ascii(), "...........................OO.OOOO.OOO..........................");
        sim.step();

        assert_eq!(sim.to_ascii(), "..........................OO..O....O..O.........................");
        sim.step();

        assert_eq!(sim.to_ascii(), ".........................OO.OOOO..OOOOOO........................");
        sim.step();

        assert_eq!(sim.to_ascii(), "........................OO..O...OOO.....O.......................");
        sim.step();

        assert_eq!(sim.to_ascii(), ".......................OO.OOOO.OO..O...OOO......................");
        sim.step();

        assert_eq!(sim.to_ascii(), "......................OO..O....O.OOOO.OO..O.....................");
        sim.step();

        assert_eq!(sim.to_ascii(), ".....................OO.OOOO..OO.O....O.OOOO....................");
        sim.step();

        assert_eq!(sim.to_ascii(), "....................OO..O...OOO..OO..OO.O...O...................");
        sim.step();

        assert_eq!(sim.to_ascii(), "...................OO.OOOO.OO..OOO.OOO..OO.OOO..................");
        sim.step();

        assert_eq!(sim.to_ascii(), "..................OO..O....O.OOO...O..OOO..O..O.................");
        sim.step();

        assert_eq!(sim.to_ascii(), ".................OO.OOOO..OO.O..O.OOOOO..OOOOOOO................");
        sim.step();

        assert_eq!(sim.to_ascii(), "................OO..O...OOO..OOOO.O....OOO......O...............");
        sim.step();

        assert_eq!(sim.to_ascii(), "...............OO.OOOO.OO..OOO....OO..OO..O....OOO..............");
        sim.step();

        assert_eq!(sim.to_ascii(), "..............OO..O....O.OOO..O..OO.OOO.OOOO..OO..O.............");
        sim.step();

        assert_eq!(sim.to_ascii(), ".............OO.OOOO..OO.O..OOOOOO..O...O...OOO.OOOO............");
        sim.step();

        assert_eq!(sim.to_ascii(), "............OO..O...OOO..OOOO.....OOOO.OOO.OO...O...O...........");
        sim.step();

        assert_eq!(sim.to_ascii(), "...........OO.OOOO.OO..OOO...O...OO....O...O.O.OOO.OOO..........");
        sim.step();

        assert_eq!(sim.to_ascii(), "..........OO..O....O.OOO..O.OOO.OO.O..OOO.OO.O.O...O..O.........");
        sim.step();

        assert_eq!(sim.to_ascii(), ".........OO.OOOO..OO.O..OOO.O...O..OOOO...O..O.OO.OOOOOO........");
        sim.step();

        assert_eq!(sim.to_ascii(), "........OO..O...OOO..OOOO...OO.OOOOO...O.OOOOO.O..O.....O.......");
        sim.step();

        assert_eq!(sim.to_ascii(), ".......OO.OOOO.OO..OOO...O.OO..O....O.OO.O.....OOOOO...OOO......");
        sim.step();

        assert_eq!(sim.to_ascii(), "......OO..O....O.OOO..O.OO.O.OOOO..OO.O..OO...OO....O.OO..O.....");
        sim.step();

        assert_eq!(sim.to_ascii(), ".....OO.OOOO..OO.O..OOO.O..O.O...OOO..OOOO.O.OO.O..OO.O.OOOO....");
        sim.step();

        assert_eq!(sim.to_ascii(), "....OO..O...OOO..OOOO...OOOO.OO.OO..OOO....O.O..OOOO..O.O...O...");
        sim.step();

        assert_eq!(sim.to_ascii(), "...OO.OOOO.OO..OOO...O.OO....O..O.OOO..O..OO.OOOO...OOO.OO.OOO..");
        sim.step();

        assert_eq!(sim.to_ascii(), "..OO..O....O.OOO..O.OO.O.O..OOOOO.O..OOOOOO..O...O.OO...O..O..O.");
        sim.step();

        assert_eq!(sim.to_ascii(), ".OO.OOOO..OO.O..OOO.O..O.OOOO.....OOOO.....OOOO.OO.O.O.OOOOOOOOO");
        sim.step();
    }
}
