use std::{io::BufRead, sync::LazyLock};

use bevy::utils::HashMap;
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    rngs::ThreadRng,
};

pub static NAMES: LazyLock<ProbabilityTable> = LazyLock::new(|| {
    ProbabilityTable::from_reader(include_bytes!("../assets/names.txt").as_ref(), 3)
});

pub static WORLD_NAMES: LazyLock<ProbabilityTable> = LazyLock::new(|| {
    ProbabilityTable::from_reader(include_bytes!("../assets/world_names.txt").as_ref(), 3)
});

#[derive(Debug, Clone)]
pub struct ProbabilityTable {
    table: HashMap<String, HashMap<char, u32>>,
    accuracy: usize,
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
                format!("{:accuracy$}{}", " ", line.to_lowercase())
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
