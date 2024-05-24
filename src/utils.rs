use bevy::math::IVec2;

#[macro_export]
macro_rules! extract_ok {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return,
        }
    };
}

#[macro_export]
macro_rules! extract_some {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
}

pub fn manhattan_distance(a: IVec2, b: IVec2) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

pub struct Map2D<T: Copy> {
    data: Vec<Vec<T>>,
    size: usize,
}

impl<T: Copy> Map2D<T> {
    pub fn new(fill_with: T, n: usize) -> Self {
        Self {
            data: vec![vec![fill_with; n]; n],
            size: n,
        }
    }

    pub fn get(&self, index: IVec2) -> Option<T> {
        if index.x < 0 || index.y < 0 {
            return None;
        }

        let x = index.x as usize;
        let y = index.y as usize;

        if x >= self.size || y >= self.size {
            return None;
        }

        Some(self.data[x][y])
    }

    pub fn set(&mut self, index: IVec2, value: T) {
        if index.x < 0 || index.y < 0 {
            return;
        }

        let x = index.x as usize;
        let y = index.y as usize;

        if x < self.size && y < self.size {
            self.data[x][y] = value;
        }
    }
}
