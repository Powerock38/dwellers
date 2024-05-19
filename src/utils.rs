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

pub struct BoolMap<const N: usize> {
    data: [[bool; N]; N],
}

impl<const N: usize> BoolMap<N> {
    pub fn new() -> Self {
        Self {
            data: [[false; N]; N],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        if x >= N || y >= N {
            return false;
        }
        self.data[x][y]
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        if x < N && y < N {
            self.data[x][y] = value;
        }
    }
}
