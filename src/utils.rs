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

#[macro_export]
macro_rules! enum_map {
    ($enum_name:ident => $tile_data_type:ty {
        $first_name:ident = $first_data:expr,
        $( $name:ident = $data:expr ),* $(,)?
    }) => {
        #[derive(PartialEq, Eq, Hash, Clone, Copy, Reflect, Default, Encode, Decode, Debug)]
        pub enum $enum_name {
            #[default]
            $first_name,
            $(
                $name,
            )*
        }

        impl $enum_name {
            #[inline]
            pub fn data(&self) -> $tile_data_type {
                match self {
                    Self::$first_name => $first_data,
                    $(
                        Self::$name => $data,
                    )*
                }
            }
        }
    }
}
