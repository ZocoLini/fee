use std::{iter::Peekable, str::CharIndices};

pub fn parse_uf64(c: char, chars: &mut Peekable<CharIndices>) -> f64
{
    let mut value: f64 = 0.0;
    let mut frac = 0.1;
    let mut is_fraction = false;

    match c {
        '0'..='9' => {
            if is_fraction {
                value += (c as u8 - b'0') as f64 * frac;
                frac *= 0.1;
            } else {
                value = value * 10.0 + (c as u8 - b'0') as f64;
            }
        }
        '.' if !is_fraction => {
            is_fraction = true;
        }
        _ => unreachable!("can't happend"),
    }

    while let Some(&(_, d)) = chars.peek() {
        match d {
            '0'..='9' => {
                chars.next();
                if is_fraction {
                    value += (d as u8 - b'0') as f64 * frac;
                    frac *= 0.1;
                } else {
                    value = value * 10.0 + (d as u8 - b'0') as f64;
                }
            }
            '.' if !is_fraction => {
                chars.next();
                is_fraction = true;
            }
            _ => break,
        }
    }

    value
}

pub fn parse_usize(s: &[u8]) -> usize
{
    let mut result = 0;

    for &byte in s {
        result = result * 10 + (byte - b'0');
    }

    result as usize
}
