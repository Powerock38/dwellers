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

#[macro_export]
macro_rules! enum_map {
    ($enum_name:ident => $data_type:ty {
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
            pub fn data(&self) -> $data_type {
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

#[inline]
pub fn div_to_floor(a: IVec2, b: IVec2) -> IVec2 {
    let mut result = a / b;
    if a.x % b.x != 0 && a.x < 0 {
        result.x -= 1;
    }
    if a.y % b.y != 0 && a.y < 0 {
        result.y -= 1;
    }
    result
}
