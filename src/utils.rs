use std::path::Path;

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

pub fn write_to_file(path: String, content: impl AsRef<[u8]>) {
    let path = Path::new(&path);
    path.parent()
        .and_then(|p| std::fs::create_dir_all(p).ok())
        .expect("Error while creating parent directory");

    std::fs::write(path, content).expect("Error while writing to file");
}

pub fn pascal_case_to_title_case(input: &str) -> String {
    let mut result = String::new();

    for (i, c) in input.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            result.push(' ');
        }
        result.push(c);
    }

    result
}
