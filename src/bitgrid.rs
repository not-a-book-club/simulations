use crate::Grid;

use alloc::vec;
use alloc::vec::Vec;

#[derive(Clone, PartialEq, Eq)]
pub struct BitGrid {
    buf: Vec<u8>,
    width: i16,
    height: i16,
}

impl core::fmt::Debug for BitGrid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BitGrid")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("set/unset", &(self.count_set(), self.count_unset()))
            .finish()
    }
}

impl BitGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let buf = vec![0; width.div_ceil(8) * height];

        Self {
            buf,
            width: width as i16,
            height: height as i16,
        }
    }

    pub fn new_with_fn<F>(width: usize, height: usize, mut func: F) -> Self
    where
        F: FnMut(i16, i16) -> bool,
    {
        let mut grid = Self::new(width, height);
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                grid.set(x, y, func(x, y));
            }
        }
        grid
    }

    pub fn parse<const N: usize>(text: &str, set: [char; N]) -> Option<Self> {
        let dim_y = text.lines().count() - 1;
        let dim_x = text.lines().next().map(|l| l.len() - 1).unwrap_or(0);

        let mut grid = Self::new(dim_x, dim_y);
        let mut y = 0;
        for line in text.lines() {
            let (line, _) = line.split_once("#").unwrap_or((line, ""));

            if line.is_empty() {
                continue;
            }
            for (x, c) in line.chars().enumerate() {
                if set.contains(&c) {
                    grid.set(x as _, y as _, true);
                }
            }
            y += 1;
        }

        Some(grid)
    }

    pub fn width(&self) -> i16 {
        self.width
    }

    pub fn height(&self) -> i16 {
        self.height
    }

    pub fn dims(&self) -> (i16, i16) {
        (self.width(), self.height())
    }

    pub fn is_empty(&self) -> bool {
        self.buf.iter().all(|&byte| byte == 0)
    }

    pub fn count_set(&self) -> usize {
        self.buf
            .iter()
            .map(|&byte| byte.count_ones() as usize)
            .sum()
    }

    pub fn count_unset(&self) -> usize {
        self.buf
            .iter()
            .map(|&byte| byte.count_zeros() as usize)
            .sum()
    }

    #[track_caller]
    pub fn get(&self, x: i16, y: i16) -> bool {
        let (idx, bit) = self.idx(x, y);
        let mask = 1 << bit;

        (self.buf[idx] & mask) != 0
    }

    #[track_caller]
    pub fn set(&mut self, x: i16, y: i16, elem: bool) -> bool {
        let (idx, bit) = self.idx(x, y);
        let mask = 1 << bit;

        let old = (self.buf[idx] & mask) != 0;

        self.buf[idx] &= !mask;
        self.buf[idx] |= (elem as u8) << bit;

        old
    }

    #[track_caller]
    pub fn flip(&mut self, x: i16, y: i16) -> bool {
        let (idx, bit) = self.idx(x, y);
        let mask = 1 << bit;

        let old = (self.buf[idx] & mask) != 0;

        self.buf[idx] ^= 1 << bit;

        old
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.buf
    }

    pub fn idx(&self, mut x: i16, mut y: i16) -> (usize, u8) {
        // Wrap x and y along their axis
        x = (x + self.width()) % self.width();
        y = (y + self.height()) % self.height();

        let x = x as usize;
        let y = y as usize;

        let idx = (x / 8) + y * (self.width() as usize).div_ceil(8);
        let bit = x % 8;

        (idx, bit as u8)
    }

    pub fn diff_with(&self, other: &BitGrid) -> BitGrid {
        assert_eq!(self.width(), other.width());
        assert_eq!(self.height(), other.height());

        let mut diff = Self::new(self.width() as _, self.height() as _);
        let bytes = diff.as_mut_bytes();

        for (i, (a, b)) in self.as_bytes().iter().zip(other.as_bytes()).enumerate() {
            bytes[i] = a ^ b;
        }

        diff
    }
}

impl Grid for BitGrid {
    fn new(width: usize, height: usize) -> Self {
        Self::new(width, height)
    }

    fn width(&self) -> i16 {
        self.width()
    }

    fn height(&self) -> i16 {
        self.height()
    }

    #[track_caller]
    fn get(&self, x: i16, y: i16) -> bool {
        self.get(x, y)
    }

    #[track_caller]
    fn set(&mut self, x: i16, y: i16, set: bool) -> bool {
        self.set(x, y, set)
    }

    #[track_caller]
    fn flip(&mut self, x: i16, y: i16) -> bool {
        self.flip(x, y)
    }

    fn fill(&mut self, set: bool) {
        if set {
            self.as_mut_bytes().fill(0b0000_0000_u8);
        } else {
            self.as_mut_bytes().fill(0b1111_1111_u8);
        }
    }
}

/// Local typedef to simplify using image's image type. We won't be changing the backing store from a Vec.
#[cfg(feature = "image")]
type ImageBuffer<P> = image::ImageBuffer<P, Vec<<P as image::Pixel>::Subpixel>>;

#[cfg(feature = "image")]
impl BitGrid {
    /// Convert the bitgrid into an [`image::ImageBuffer`](image::ImageBuffer).
    ///
    /// # Coloring
    /// - `palette[0]` is the pixel color used for set cells.
    /// - `palette[1]` is the pixel color used for unset cells.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use simulations::BitGrid;
    /// # use image::*;
    /// // Build a basic bitgrid: All '1' cells are set, everything else is unset.
    /// let bitgrid: BitGrid = BitGrid::parse("1000\n01000\n0010\n0001", ['1']).unwrap();
    ///
    /// // Save it as an 8-bit Grayscale PNG
    /// bitgrid
    ///     .to_image([
    ///         Luma([0xff_u8]),
    ///         Luma([0x00_u8]),
    ///     ])
    ///     .save("example_gray.png")
    ///     .expect("Failed to save image");
    ///
    /// // Save it as an 8-bit RGB PNG
    /// bitgrid
    ///     .to_image::<Rgb<u8>>([
    ///         Rgb([0xff, 0x00, 0xff]),
    ///         Rgb([0x00, 0x00, 0x00]),
    ///     ])
    ///     .save("example_rgb.png")
    ///     .expect("Failed to save image");  
    /// ```
    pub fn to_image<P>(&self, palette: [P; 2]) -> ImageBuffer<P>
    where
        P: image::Pixel,
    {
        ImageBuffer::<P>::from_fn(
            self.width() as u32,
            self.height() as u32,
            |x: u32, y: u32| {
                if self.get(x as _, y as _) {
                    palette[0]
                } else {
                    palette[1]
                }
            },
        )
    }

    /// Convert the bitgrid into an [`image::ImageBuffer`](image::ImageBuffer).
    ///
    /// # Coloring
    /// This method works like [`to_image`](Self::to_image), except with a provided grayscale palette.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use simulations::BitGrid;
    /// # use image::*;
    /// // Build a basic bitgrid: All '1' cells are set, everything else is unset.
    /// let bitgrid: BitGrid = BitGrid::parse("1000\n01000\n0010\n0001", ['1']).unwrap();
    ///
    /// // Save it as an 8-bit Grayscale PNG
    /// bitgrid
    ///     .to_image_grayscale()
    ///     .save("example_gray.png")
    ///     .expect("Failed to save image");
    ///
    pub fn to_image_grayscale(&self) -> ImageBuffer<image::Luma<u8>> {
        use image::Luma;

        self.to_image([
            Luma([0xff_u8]), // set   Cells
            Luma([0x00_u8]), // unset Cells
        ])
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case, clippy::bool_assert_comparison)]
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rstest::*;

    #[test]
    fn check_0x0() {
        // Make sure things don't panic
        let grid = BitGrid::new(0, 0);
        dbg!(&grid);

        let bytes = grid.as_bytes();
        dbg!(bytes);
    }

    #[rstest]
    #[case::x_is_00(0, (0, 0))]
    #[case::x_is_01(1, (0, 1))]
    #[case::x_is_04(4, (0, 4))]
    #[case::x_is_08(8, (1, 0))]
    #[case::x_is_12(12, (1, 4))]
    #[case::x_is_16(16, (2, 0))]
    #[case::x_is_17(17, (2, 1))]
    // Check wrapping behavior too
    #[case::x_is_00_wrap(0+32, (0, 0))]
    #[case::x_is_01_wrap(1+32, (0, 1))]
    #[case::x_is_04_wrap(4+32, (0, 4))]
    #[case::x_is_08_wrap(8+32, (1, 0))]
    #[case::x_is_12_wrap(12+32, (1, 4))]
    #[case::x_is_16_wrap(16+32, (2, 0))]
    #[case::x_is_17_wrap(17+32, (2, 1))]
    fn check_32x1_idx(#[case] x: i16, #[case] (idx, bit): (usize, u8)) {
        let grid = BitGrid::new(32, 1);
        let y = 0;

        println!("Checking index of x={x}, y={y}");
        let ans = grid.idx(x, y);
        let expected = (idx, bit);
        assert_eq!(
            ans, expected,
            "Flat index of ({x}, {y}) was {ans:?} but should have been {expected:?}"
        );

        // Make sure this doesn't panic
        let _ = grid.get(x, y);
    }

    #[rstest]
    #[case::y_is_00(0, (0, 0))]
    #[case::y_is_01(1, (1, 0))]
    #[case::y_is_04(4, (4, 0))]
    #[case::y_is_08(8, (8, 0))]
    #[case::y_is_12(12, (12, 0))]
    #[case::y_is_16(16, (16, 0))]
    #[case::y_is_17(17, (17, 0))]
    // Check wrapping behavior too
    #[case::y_is_00_wrap(0+32, (0, 0))]
    #[case::y_is_01_wrap(1+32, (1, 0))]
    #[case::y_is_04_wrap(4+32, (4, 0))]
    #[case::y_is_08_wrap(8+32, (8, 0))]
    #[case::y_is_12_wrap(12+32, (12, 0))]
    #[case::y_is_16_wrap(16+32, (16, 0))]
    #[case::y_is_17_wrap(17+32, (17, 0))]
    fn check_1x32_idx(#[case] y: i16, #[case] (idx, bit): (usize, u8)) {
        let grid = BitGrid::new(1, 32);
        let x = 0;

        println!("Checking index of x={x}, y={y}");
        let ans = grid.idx(x, y);
        let expected = (idx, bit);
        assert_eq!(
            ans, expected,
            "Flat index of ({x}, {y}) was {ans:?} but should have been {expected:?}"
        );

        // Make sure this doesn't panic
        let _ = grid.get(x, y);
    }

    #[test]
    fn check_parse_diagonal() {
        let text = indoc!(
            r#"# 4x4
            O...
            .X..
            ..O. # This comment should be ignored
            ...O
            "#
        );
        println!("text={text}");
        let maybe_grid = BitGrid::parse(text, ['O', 'X']);

        let mut expected = BitGrid::new(4, 4);
        expected.set(0, 0, true);
        expected.set(1, 1, true);
        expected.set(2, 2, true);
        expected.set(3, 3, true);

        assert_eq!(maybe_grid, Some(expected));
    }

    #[test]
    fn check_parse_diagonal_rev() {
        let text = indoc!(
            r#"# 4x4
               O
              O
             X
            O
            "#
        );
        println!("text={text}");
        let maybe_grid = BitGrid::parse(text, ['O', 'X']);

        let mut expected = BitGrid::new(4, 4);
        expected.set(3, 0, true);
        expected.set(2, 1, true);
        expected.set(1, 2, true);
        expected.set(0, 3, true);

        assert_eq!(maybe_grid, Some(expected));
    }

    #[test]
    fn check_get_set() {
        let mut grid = BitGrid::new(16, 16);
        assert!(grid.is_empty());

        for y in 0..grid.height() {
            for x in 0..grid.width() {
                assert!(grid.is_empty());
                assert_eq!(grid.get(x, y), false);

                grid.set(x, y, true);
                assert!(!grid.is_empty());
                assert_eq!(grid.get(x, y), true);

                grid.set(x, y, false);
                assert_eq!(grid.get(x, y), false);
            }
        }
    }

    #[test]
    fn check_flip() {
        let mut grid = BitGrid::new(16, 16);
        assert!(grid.is_empty());

        for y in 0..grid.height() {
            for x in 0..grid.width() {
                grid.flip(x, y);
            }
        }

        assert_eq!(grid.is_empty(), false);
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                assert_eq!(grid.get(x, y), true);
            }
        }
    }

    #[test]
    fn check_byte_layout() {
        let mut grid = BitGrid::new(16, 16);

        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let (idx, bit) = grid.idx(x, y);
                println!("Checking setting bit at ({x}, {y}) ~= idx={idx}, bit={bit}");
                assert_eq!(grid.get(x, y), false, "Failed to get bit at ({x}, {y})");
                grid.set(x, y, true);
            }
        }

        let byte_len = (grid.width() * grid.height() / 8) as usize;
        assert_eq!(grid.as_bytes().len(), byte_len);
        assert_eq!(grid.as_bytes(), vec![0b1111_1111; byte_len]);
    }
}
