use std::{io::BufRead, sync::LazyLock};

use bevy::utils::HashMap;
use rand::{distributions::WeightedIndex, prelude::*};

pub static NAMES: LazyLock<ProbabilityTable> = LazyLock::new(|| {
    ProbabilityTable::from_reader(include_bytes!("../assets/names.txt").as_ref(), 3)
});

#[derive(Debug, Clone)]
pub struct ProbabilityTable {
    pub(crate) table: HashMap<String, HashMap<char, u32>>,
    pub(crate) accuracy: usize,
}

impl ProbabilityTable {
    fn new(accuracy: usize) -> ProbabilityTable {
        ProbabilityTable {
            table: HashMap::new(),
            accuracy,
        }
    }

    pub fn from_reader<T: BufRead>(reader: T, accuracy: usize) -> ProbabilityTable {
        assert!(accuracy >= 1);
        generate_table(add_space(reader, accuracy), accuracy)
    }
}

// Replace each new line characters by a series of space of length
fn add_space<T: BufRead>(reader: T, accuracy: usize) -> String {
    reader
        .lines()
        .map(|line| {
            line.map_or(String::new(), |line| {
                format!(
                    "{:accuracy$}{}",
                    " ",
                    line.to_lowercase(),
                    accuracy = accuracy
                )
            })
        })
        .collect()
}

// Generate a ProbabilityTable from the output of add_space
fn generate_table(spaced_file: String, accuracy: usize) -> ProbabilityTable {
    let mut table = ProbabilityTable::new(accuracy);
    let chars_list: Vec<_> = spaced_file.chars().collect();
    for charactere in 0..chars_list.len() - accuracy {
        let key: String = chars_list
            .get(charactere..charactere + accuracy)
            .unwrap()
            .iter()
            .collect();

        let value: char = *chars_list.get(charactere + accuracy).unwrap();

        *table
            .table
            .entry(key)
            .or_default()
            .entry(value)
            .or_default() += 1;
    }
    table
}

// Generate one word from a ProbabilityTable
pub fn generate_word(table: &ProbabilityTable, rng: &mut ThreadRng) -> String {
    let mut out = " ".repeat(table.accuracy);
    loop {
        let chars_list: Vec<_> = out.chars().collect();
        let key = &chars_list[chars_list.len() - table.accuracy..]
            .iter()
            .collect::<String>();
        let choices = table.table.get(key).unwrap();
        let weight = WeightedIndex::new(choices.values()).unwrap();
        let next_letter = choices.keys().collect::<Vec<&char>>()[weight.sample(rng)];
        out += &next_letter.to_string();
        if out.ends_with(' ') {
            break;
        }
    }
    out.trim().to_string()
}

// fn generate_multiple_words(matrix: &ProbabilityTable, number: u32) -> Vec<String> {
//     let mut vec_string = Vec::new();
//     let mut rng = thread_rng();
//     for _ in 0..number {
//         vec_string.push(generate_word(matrix, &mut rng));
//     }
//     vec_string
// }

// pub fn generate_words<T: BufRead>(reader: T, accuracy: usize, amout: u32) -> Vec<String> {
//     let mut out = generate_multiple_words(
//         &generate_table(add_space(reader, accuracy), accuracy),
//         amout,
//     );
//     out.sort_by_key(String::len);
//     out
// }
