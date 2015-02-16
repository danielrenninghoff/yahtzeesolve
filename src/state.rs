/*
Copyright (c) 2015, Daniel Renninghoff
All rights reserved.

Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its contributors
   may be used to endorse or promote products derived from this software without
   specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/
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
