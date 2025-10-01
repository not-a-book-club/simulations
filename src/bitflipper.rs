use crate::prelude::*;

pub struct BitFlipper<G: Grid = crate::BitGrid> {
    pos: IVec3,
    dir: IVec3,
    dir_sign: IVec3,
    grid: G,
}

impl<G: GridNew> BitFlipper<G> {
    pub fn new(dims: IVec3, dir: IVec3) -> Self {
        let grid = G::new(dims);
        Self::new_with_grid(grid, dir)
    }
}

// Public methods
impl<G: Grid> BitFlipper<G> {
    pub fn new_with_grid(grid: G, dir: IVec3) -> Self {
        Self {
            pos: IVec3::zero(),
            dir,
            dir_sign: IVec3::one(),
            grid,
        }
    }

    /// Destroy this BitFlipper sim and get just its backing `Grid` back
    pub fn into_grid(self) -> G {
        self.grid
    }

    /// Borrow the backing `Grid` object
    pub fn grid(&self) -> &G {
        &self.grid
    }

    /// Borrow the backing `Grid` object
    pub fn grid_mut(&mut self) -> &mut G {
        &mut self.grid
    }

    pub fn dims(&self) -> IVec3 {
        self.grid.dims()
    }

    pub fn dir(&self) -> IVec3 {
        self.dir
    }

    pub fn pos(&self) -> IVec3 {
        self.pos
    }

    pub fn resize(&mut self, new_dims: IVec3) {
        self.grid.resize(new_dims);
    }

    pub fn set_dir(&mut self, new_dir: IVec3) {
        // Sloppily approximate our current location in the new coordinate space
        let [x, y, z] = self.pos.as_array().map(|a| a.unsigned_abs().max(1) as u64);
        let [dx0, dy0, dz0] = self.dir.as_array().map(|a| a.unsigned_abs().max(1) as u64);
        let [dx1, dy1, dz1] = new_dir.as_array().map(|a| a.unsigned_abs().max(1) as u64);
        self.pos = IVec3 {
            x: (x * dy1 * dz1 / dy0 / dz0) as i32,
            y: (y * dx1 * dz1 / dx0 / dz0) as i32,
            z: (z * dx1 * dy1 / dx0 / dy0) as i32,
        };

        self.dir = new_dir;
    }

    /// Flip and advance the sim `dir.abs()` times. If `dir` is negative, the sim runs backwards.
    pub fn step(&mut self, dir: i32) {
        for _ in 0..dir.abs() {
            self.flip_and_advance_once(dir.signum());
        }
    }
}

// Core Bitflipper Logic
impl<G: Grid> BitFlipper<G> {
    fn flip_and_advance_once(&mut self, dir: i32) {
        debug_assert!(dir == dir.signum());

        if self.pos.x <= 0 {
            self.dir_sign.x = self.dir.x.signum() * dir.signum();
        }

        if self.pos.x >= self.grid.width() * self.dir.y.abs().max(1) * self.dir.z.abs().max(1) {
            self.dir_sign.x = -self.dir.x.signum() * dir.signum();
        }

        if self.pos.y <= 0 {
            self.dir_sign.y = self.dir.y.signum() * dir.signum();
        }

        if self.pos.y >= self.grid.height() * self.dir.x.abs().max(1) * self.dir.z.abs().max(1) {
            self.dir_sign.y = -self.dir.y.signum() * dir.signum();
        }

        if self.pos.z <= 0 {
            self.dir_sign.z = self.dir.z.signum() * dir.signum();
        }

        if self.pos.z >= self.grid.depth() * self.dir.x.abs().max(1) * self.dir.y.abs().max(1) {
            self.dir_sign.z = -self.dir.z.signum() * dir.signum();
        }

        self.flip_bit(dir.signum());

        let next_x = Self::next_multiple_of_n_in_direction(
            self.pos.x,
            self.dir.y.abs().max(1) * self.dir.z.abs().max(1),
            self.dir.x * self.dir_sign.x * dir,
        );
        let next_y = Self::next_multiple_of_n_in_direction(
            self.pos.y,
            self.dir.x.abs().max(1) * self.dir.z.abs().max(1),
            self.dir.y * self.dir_sign.y * dir,
        );
        let next_z = Self::next_multiple_of_n_in_direction(
            self.pos.z,
            self.dir.x.abs().max(1) * self.dir.y.abs().max(1),
            self.dir.z * self.dir_sign.z * dir,
        );

        let dist_x = (next_x - self.pos.x).abs();
        let dist_y = (next_y - self.pos.y).abs();
        let dist_z = (next_z - self.pos.z).abs();

        let mut move_amount = i32::MAX;

        if dist_x > 0 && dist_x < move_amount {
            move_amount = dist_x;
        }
        if dist_y > 0 && dist_y < move_amount {
            move_amount = dist_y;
        }
        if dist_z > 0 && dist_z < move_amount {
            move_amount = dist_z;
        }

        self.pos.x += move_amount * dir * self.dir.x.signum() * self.dir_sign.x;
        self.pos.y += move_amount * dir * self.dir.y.signum() * self.dir_sign.y;
        self.pos.z += move_amount * dir * self.dir.z.signum() * self.dir_sign.z;
    }

    fn next_multiple_of_n_in_direction(i: i32, n: i32, dir: i32) -> i32 {
        if dir < 0 {
            return -Self::next_multiple_of_n_in_direction(-i, -n, -dir);
        }

        i + n.abs() - Self::positive_modulo(i, n)
    }

    fn positive_modulo(i: i32, n: i32) -> i32 {
        (n.abs() + (i % n.abs())) % n.abs()
    }

    // Flip the pixel we're about to traverse.
    fn flip_bit(&mut self, dir: i32) {
        // When we're on an edge, it's ambigious which pixel we should flip.
        // So subtract 1 when moving in the negative direction to move within the target pixel.
        //     .->
        // |---|---|
        //       ^   flip this
        //
        //   <-.
        // |---|---|
        //   ^       flip that

        let mut pos: IVec3 = self.pos;
        if dir * self.dir.x * self.dir_sign.x < 0 {
            pos.x -= 1;
        }
        if dir * self.dir.y * self.dir_sign.y < 0 {
            pos.y -= 1;
        }
        if dir * self.dir.z * self.dir_sign.z < 0 {
            pos.z -= 1;
        }

        // We're dividing by dir.abs(), but need to handle a possible 0.
        let dir: IVec3 = self.dir.abs().max_by_component(IVec3::one());

        let x: Index = pos.x / dir.y / dir.z;
        let y: Index = pos.y / dir.x / dir.z;
        let z: Index = pos.z / dir.x / dir.y;

        self.grid.flip(x, y, z);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // All the tests use BitGrid with BitFlipper
    type BitFlipper = crate::BitFlipper<BitGrid>;

    use pretty_assertions::assert_eq;
    use rstest::*;

    #[rstest]
    #[case::simple_1_to_2(1, 1, 1, 2)]
    #[case::simple_2_to_4(2, 2, 2, 4)]
    #[case::simple_3_to_4(3, 2, 2, 4)]
    #[case::simple_3_to_7(3, 7, 2, 7)]
    #[case::simple_9_to_12(9, 3, 3, 12)]
    #[case::simple_9_to_5(9, 5, -1, 5)]
    #[case::simple_negative_9_to_negative_5(-9, -5, 1, -5)]
    #[case::simple_negative_9_to_negative_10(-9, -10, -1, -10)]
    fn test_next_multiple_of_n_in_direction(
        #[case] i: i32,
        #[case] n: i32,
        #[case] dir: i32,
        #[case] expected: i32,
    ) {
        assert_eq!(
            expected,
            BitFlipper::next_multiple_of_n_in_direction(i, n, dir)
        );
    }

    #[test]
    fn test_1_by_1_by_1_enabled() {
        let mut expected = BitGrid::new(1, 1, 1);
        expected.flip(1, 1, 1);

        let mut bit_flipper = BitFlipper::new(expected.dims(), IVec3::one());
        bit_flipper.step(1);

        let actual: &_ = bit_flipper.grid();
        save_test_image("simple_1_by_1", "expected", &expected);
        save_test_image("simple_1_by_1", "actual", actual);

        assert_eq!(&expected, actual);
    }

    #[test]
    fn test_32_by_32_by_32_simple_diagonal() {
        let expected = BitGrid::new(32, 32, 32);

        let mut bit_flipper = BitFlipper::new(expected.dims(), IVec3::one());
        for _i in 0..64 {
            bit_flipper.step(1);
        }

        let actual: &_ = bit_flipper.grid();
        save_test_image("simple_32_by_32", "expected", &expected);
        save_test_image("simple_32_by_32", "actual", actual);

        assert_eq!(&expected, actual);
    }

    fn save_test_image(scope: &str, label: &str, frame: &BitGrid) {
        use image::imageops;
        eprintln!("+ Saving {scope}_{label}:");

        // TODO: Should probably sanitize scope incase it contains "::" or something that makes for bad filenames.

        // Usually the folder with the Cargo.toml
        eprintln!(
            "+ Running from {}",
            std::env::current_dir().unwrap_or_default().display()
        );

        let out_dir = "./target/test-images";
        std::fs::create_dir_all(out_dir).unwrap();
        let out_path = format!("{out_dir}/{scope}_{label}.png");
        eprintln!(
            "+ Saving to {out_path} ({}x{})",
            frame.width(),
            frame.height()
        );

        let mut img = frame.to_image_grayscale();
        let max_dim = i32::max(i32::max(frame.width(), frame.height()), frame.depth()) as f32;

        // Make it readable
        if max_dim < 500. {
            let nw = (img.width() as f32 * (500. / max_dim)) as u32;
            let nh = (img.height() as f32 * (500. / max_dim)) as u32;
            img = imageops::resize(&img, nw, nh, imageops::FilterType::Nearest);
        }

        img.save(out_path).unwrap();
    }
}
