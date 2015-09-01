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

pub fn upper_score(roll: &[u8;6], cat: u8) -> u8 {
    match cat {
        0 ... 5 => {
            return roll[cat as usize] * (cat + 1);
        }
        _ => {
            return 0;
        }
    }
}

pub fn score(roll: &[u8;6], cat: u8) -> u8 {
    match cat {
        0 ... 5 => {
            return roll[cat as usize] * (cat + 1);
        },
        6 => {
            if roll.iter().fold(false, |a, &b| a || (b >= 3)) {
                let mut tmp = 0;
                for i in 0..6 {
                    tmp += roll[i] * ((i as u8) + 1)
                }
                return tmp;
            }
            else {
                return 0;
            }
        }
        7 => {
            if roll.iter().fold(false, |a, &b| a || (b >= 4)) {
                let mut tmp = 0;
                for i in 0..6 {
                    tmp += roll[i] * ((i as u8) + 1)
                }
                return tmp;
            }
            else {
                return 0;
            }
        }
        8 => {
            if roll.contains(&3) && roll.contains(&2) {
                return 25;
            }
            else {
                return 0;
            }
        }
        9 => {
            if    (roll[0] >= 1 && roll[1] >= 1 && roll[2] >= 1 && roll[3] >= 1)
               || (roll[1] >= 1 && roll[2] >= 1 && roll[3] >= 1 && roll[4] >= 1)
               || (roll[2] >= 1 && roll[3] >= 1 && roll[4] >= 1 && roll[5] >= 1) {
                   return 30;
               }
            else {
                return 0;
            }
        }
        10 => {
            if    (roll[0] == 1 && roll[1] == 1 && roll[2] == 1 && roll[3] == 1 && roll[4] == 1)
               || (roll[1] == 1 && roll[2] == 1 && roll[3] == 1 && roll[4] == 1 && roll[5] == 1) {
                   return 40;
               }
            else {
                return 0;
            }
        }
        11 => {
            if roll.contains(&5) {
                return 50;
            }
            else {
                return 0;
            }
        }
        12 => {
            let mut tmp = 0;
            for i in 0..6 {
                tmp += roll[i] * ((i as u8) + 1)
            }
            return tmp;
        }
        _ => {
            return 0;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score() {
        assert!(score(&[5,0,0,0,0,0], 0) == 5);
        assert!(score(&[0,5,0,0,0,0], 0) == 0);
        assert!(score(&[3,2,0,0,0,0], 6) == 7);
        assert!(score(&[2,2,1,0,0,0], 6) == 0);
        assert!(score(&[4,1,0,0,0,0], 7) == 6);
        assert!(score(&[3,2,0,0,0,0], 7) == 0);
        assert!(score(&[3,2,0,0,0,0], 8) == 25);
        assert!(score(&[2,2,1,0,0,0], 8) == 0);
        assert!(score(&[1,1,2,1,0,0], 9) == 30);
        assert!(score(&[1,1,0,1,2,0], 9) == 0);
        assert!(score(&[1,1,1,1,1,0], 10) == 40);
        assert!(score(&[1,1,1,1,0,1], 10) == 0);
        assert!(score(&[5,0,0,0,0,0], 11) == 50);
        assert!(score(&[4,0,0,1,0,0], 11) == 0);
        assert!(score(&[1,1,1,1,1,0], 12) == 15);
    }

    #[test]
    fn test_upper_score() {
        assert!(upper_score(&[0,0,5,0,0,0], 2) == 15);
        assert!(upper_score(&[0,0,5,0,0,0], 7) == 0);
    }
}
