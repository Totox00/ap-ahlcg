#![feature(ascii_char)]

use std::{
    cmp::{Ordering, max, min},
    fmt::Display,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Code {
    number: u16,
    letter: u8,
}

impl Code {
    pub const fn i64(self) -> i64 {
        self.number as i64 | (self.letter as i64) << 16
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(val: &str) -> Self {
        let mut number = 0;
        let mut letter = 0;

        let mut chars = val.as_ascii().expect("Card codes must be ascii");

        if let Some(char) = chars.last()
            && char.to_u8() >> 5 == 3
        {
            letter = char.to_u8();
            chars = &chars[0..chars.len() - 1]
        }

        for char in chars {
            number = number * 10 + (char.to_u8() - b'0') as u16;
        }

        Code { number, letter }
    }

    pub fn from_range(val: &str) -> Vec<Self> {
        if let Some((start, end)) = val.split_once('-') {
            let start = Self::from_str(start);
            let end = Self::from_str(end);

            if start.letter != end.letter {
                panic!("Illegal code range");
            }

            (min(start.number, end.number)..=max(start.number, end.number))
                .map(|number| Code { number, letter: start.letter })
                .collect()
        } else {
            vec![Self::from_str(val)]
        }
    }

    pub fn from_list(val: &str) -> impl Iterator<Item = Self> {
        val.split(' ').flat_map(Self::from_range)
    }

    pub const fn from(val: i64) -> Self {
        Self {
            number: (val & 0xffff) as u16,
            letter: (val >> 16 & 0xff) as u8,
        }
    }
}

impl PartialOrd for Code {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Code {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.number.cmp(&other.number) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.letter.cmp(&other.letter)
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:05}", self.number)?;

        if self.letter > 0 {
            write!(f, "{}", (0b011 | self.letter) as char)?;
        };

        Ok(())
    }
}
