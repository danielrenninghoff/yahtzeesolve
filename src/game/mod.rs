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

use std::cmp;

pub mod lookuptable;
pub mod generators;
pub mod rules;

#[derive(Copy,Clone,PartialEq)]
pub struct Game(pub u32);

impl Game {
    pub fn new() -> Game {
        Game(0b000_0000_0000_0000)
    }
    pub fn next_turn(self, roll: &[u8;6], cat: u8) -> Game {
        let Game(state) = self;
        let mut tmp = state | (1 << (18 - cat));
        let curr_upper = state & (0b11_1111);
        let next_upper = cmp::min(63, curr_upper + rules::upper_score(roll, cat) as u32);
        tmp |= next_upper;
        Game(tmp)
    }

    pub fn is_free(self, field: u8) -> bool {
        let Game(state) = self;
        ((state & (1 << (18 - field))) >> (18 - field)) == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_turn() {
        assert!(Game(0b011_1111_1111_1100_0000).next_turn(&[5,0,0,0,0,0], 0) == Game(0b111_1111_1111_1100_0101));
        assert!(Game(0b011_1111_1111_1111_1111).next_turn(&[5,0,0,0,0,0], 0) == Game(0b111_1111_1111_1111_1111));
    }
}
