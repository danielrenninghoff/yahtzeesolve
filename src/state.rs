use std::cmp::Ordering::{Less, Equal, Greater};
use std::cmp::Ordering;
use std::cmp;

#[derive(Clone,Eq,Debug)]
pub struct State {
    pub fields: [bool; 13],
    pub upper: u8,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..13us {
            if self.fields[i] != other.fields[i] {
                return false;
            }
        }
        return self.upper == other.upper;
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for i in 0..13us {
            if self.fields[i] && !other.fields[i] {
                return Some(Less);
            }
            else if !self.fields[i] && other.fields[i] {
                return Some(Greater);
            }
        }
        if self.upper > other.upper {
            return Some(Less);
        }
        else if self.upper < other.upper {
            return Some(Greater);
        }
        return Some(Equal);
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in 0..13us {
            if self.fields[i] && !other.fields[i] {
                return Less;
            }
            else if !self.fields[i] && other.fields[i] {
                return Greater;
            }
        }
        if self.upper > other.upper {
            return Less;
        }
        else if self.upper < other.upper {
            return Greater;
        }
        return Equal;
    }
}

pub fn new_state(state: &State, roll: &[u8;6], cat: usize) -> State {
    let mut tmp = state.clone();
    tmp.fields[cat] = true;
    tmp.upper = cmp::min(63, tmp.upper + upper_score(roll, cat));
    tmp
}

fn upper_score(roll: &[u8;6], cat: usize) -> u8 {
    match cat {
        0 ... 5 => {
            return roll[cat] * (cat + 1) as u8;
        }
        _ => {
            return 0;
        }
    }
}
