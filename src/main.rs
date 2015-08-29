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

extern crate byteorder;

use std::collections::BTreeMap;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::fs::File;
use std::path::Path;
use std::env;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::cmp;

fn generate_dice_rolls() -> Vec<[u8; 6]> {
    let mut rolls = vec![];
    for ones in 0..6 {
        for twos in 0..(6-ones) {
            for threes in 0..(6-(ones+twos)) {
                for fours in 0..(6-(ones+twos+threes)) {
                    for fives in 0..(6-(ones+twos+threes+fours)) {
                        let sixes = 5 - (ones+twos+threes+fours+fives);
                        rolls.push([ones,twos,threes,fours,fives,sixes]);
                    }
                }
            }
        }
    }
    rolls
}

fn generate_dice_keeps() -> Vec<[u8; 6]> {
    let mut keeps = vec![];
    for ones in 0..6 {
        for twos in 0..(6-ones) {
            for threes in 0..(6-(ones+twos)) {
                for fours in 0..(6-(ones+twos+threes)) {
                    for fives in 0..(6-(ones+twos+threes+fours)) {
                        for sixes in 0..(6-(ones+twos+threes+fours+fives)) {
                            keeps.push([ones,twos,threes,fours,fives,sixes]);
                        }
                    }
                }
            }
        }
    }
    keeps
}

pub fn upper_score(roll: &[u8;6], cat: usize) -> u8 {
    match cat {
        0 ... 5 => {
            return roll[cat] * (cat + 1) as u8;
        }
        _ => {
            return 0;
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("USAGE: {} [generate|play]", &args[0]);
        }
        2 => {
            match &args[1][..] {
                "generate" => {
                    let mut lookup = vec![0f64; 524288];
                    let rollvec = generate_dice_rolls();
                    let dicekeeps = generate_dice_keeps();
                    for i in (0..524288).rev() {
                        let tmp = gen_start_prob(i, &lookup, &rollvec, &dicekeeps);
                        lookup[i as usize] = tmp;
                    }
                    write_state_file(&lookup).unwrap();
                },
                "play" => {
                    let rollvec = generate_dice_rolls();
                    let dicekeeps = generate_dice_keeps();
                    let x = read_state_file().unwrap();
                    let mut state: u32 = 0;
                    for _ in 0..13 {
                        state = calc_round(state, &x, &rollvec, &dicekeeps);
                    }
                }
                _ => {

                }
            }
        }
        _ => {
            println!("USAGE: {} [generate|play]", &args[0]);
        }
    }
}

fn write_state_file(lookup: &Vec<f64>) -> io::Result<()> {
    let file = try!(File::create(&Path::new("probs.dat")));
    let mut writer = BufWriter::new(file);
    for i in 0..524288 {
        try!(writer.write_f64::<LittleEndian>(lookup[i]));
    }
    Ok(())
}

fn read_state_file() -> io::Result<Vec<f64>> {
    let mut lookup = vec![0f64; 524288];
    let file = try!(File::open(&Path::new("probs.dat")));
    let mut reader = BufReader::new(file);
    for i in 0..524288 {
        lookup[i] = try!(reader.read_f64::<LittleEndian>());
    }
    Ok(lookup)
}

fn calc_round(state: u32, lookup: &Vec<f64>, rollvec: &Vec<[u8; 6]>, dicekeeps: &Vec<[u8; 6]>) -> u32 {
    let mut end_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec {
        let (tmp,_) = gen_end_prob(state, roll, lookup);
        end_states.insert(*roll, tmp);
    }

    let mut keep_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps {
        keep_2_states.insert(*keep, gen_keep_prob(keep, &end_states));
    }

    let mut roll_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec {
        let (tmp,_) = gen_roll_prob(roll,&[0,0,0,0,0,0], &keep_2_states);
        roll_2_states.insert(*roll, tmp);
    }

    let mut keep_1_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps {
        keep_1_states.insert(*keep, gen_keep_prob(keep, &roll_2_states));
    }

    println!("New Round. Please enter your next roll:");
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    let input: u32 = line.trim().parse().unwrap();
    let inp1 = key_conv(input);
    let (_,kroll) = gen_roll_prob(&inp1,&[0,0,0,0,0,0], &keep_1_states);
    println!("{:?}", kroll);
    println!("Please enter your 2nd roll:");
    line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    let input2: u32 = line.trim().parse().unwrap();
    let roll2 = key_conv(input2);
    let (_,kroll) = gen_roll_prob(&roll2,&kroll, &keep_2_states);
    println!("{:?}", kroll);
    println!("Please enter your 3rd roll:");
    line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    let input2: u32 = line.trim().parse().unwrap();
    let mut roll2 = key_conv(input2);
    roll2[0] += kroll[0];
    roll2[1] += kroll[1];
    roll2[2] += kroll[2];
    roll2[3] += kroll[3];
    roll2[4] += kroll[4];
    roll2[5] += kroll[5];
    let (_,choseni) = gen_end_prob(state, &roll2, lookup);
    println!("Mark {}", choseni + 1);
    let scr = score(&roll2, choseni);
    println!("Score: {}", scr);
    new_state(state, &roll2, choseni)
}

fn key_conv(input: u32) -> [u8;6] {
    let mut tmp = input;
    let mut out = [0u8; 6];
    while tmp != 0 {
        let tmp2 = (tmp % 10) as usize;
        if tmp2 > 0 && tmp2 <= 6 {
            out[tmp2 - 1] += 1;
        }
        tmp /= 10;
    }
    out
}

fn gen_start_prob(state: u32, lookup: &Vec<f64>, rollvec: &Vec<[u8; 6]>, dicekeeps: &Vec<[u8; 6]>) -> f64 {
    if state == 0b111_1111_1111_1111_1111 {
        return 35f64;
    }
    else if state > 0b111_1111_1111_1100_0000 {
        return 0f64;
    }

    let mut end_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec {
        let (tmp,_) = gen_end_prob(state, roll, lookup);
        end_states.insert(*roll, tmp);
    }

    let mut keep_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps {
        keep_2_states.insert(*keep, gen_keep_prob(keep, &end_states));
    }

    let mut roll_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec {
        let (tmp,_) = gen_roll_prob(roll,&[0,0,0,0,0,0], &keep_2_states);
        roll_2_states.insert(*roll, tmp);
    }

    let mut keep_1_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps {
        keep_1_states.insert(*keep, gen_keep_prob(keep, &roll_2_states));
    }

    let mut roll_1_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec {
        let (tmp,_) = gen_roll_prob(roll,&[0,0,0,0,0,0], &keep_1_states);
        roll_1_states.insert(*roll, tmp);
    }

    let mut sum = 0f64;
    let mut cnt = 0f64;
    for roll in rollvec {
        sum += *roll_1_states.get(roll).expect("asd");
        cnt += 1f64;
    }
    (sum / cnt)
}

fn gen_keep_prob(lroll: &[u8; 6], end_states: &BTreeMap<[u8; 6], f64>) -> f64 {
    let mut sum = 0f64;
    let mut cnt = 0f64;
    for ones in lroll[0]..6 {
        if ones+lroll[1]+lroll[2]+lroll[3]+lroll[4]+lroll[5] == 5 {
            sum += *(end_states.get(&[ones,lroll[1],lroll[2],lroll[3],lroll[4],lroll[5]]).expect("asdfy"));
            cnt += 1f64;
            break;
        }
        else {
            for twos in lroll[1]..6 {
                if ones+twos+lroll[2]+lroll[3]+lroll[4]+lroll[5] == 5 {
                    sum += *(end_states.get(&[ones,twos,lroll[2],lroll[3],lroll[4],lroll[5]]).expect("asdfy"));
                    cnt += 1f64;
                    break;
                }
                else {
                    for threes in lroll[2]..6 {
                        if ones+twos+threes+lroll[3]+lroll[4]+lroll[5] == 5 {
                            sum += *(end_states.get(&[ones,twos,threes,lroll[3],lroll[4],lroll[5]]).expect("asdfy"));
                            cnt += 1f64;
                            break;
                        }
                        else {
                            for fours in lroll[3]..6 {
                                if ones+twos+threes+fours+lroll[4]+lroll[5] == 5 {
                                    sum += *(end_states.get(&[ones,twos,threes,fours,lroll[4],lroll[5]]).expect("asdfy"));
                                    cnt += 1f64;
                                    break;
                                }
                                else {
                                    for fives in lroll[4]..6 {
                                        if ones+twos+threes+fours+fives+lroll[5] == 5 {
                                            sum += *(end_states.get(&[ones,twos,threes,fours,fives,lroll[5]]).expect("asdfy"));
                                            cnt += 1f64;
                                            break;
                                        }
                                        else {
                                            for sixes in lroll[5]..6 {
                                                if ones+twos+threes+fours+fives+sixes == 5 {
                                                    sum += *(end_states.get(&[ones,twos,threes,fours,fives,sixes]).expect("asdfy"));
                                                    cnt += 1f64;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    sum / cnt
}

fn gen_end_prob(state: u32, lroll: &[u8; 6], lookup: &Vec<f64>) -> (f64, usize) {
    let mut max = 0f64;
    let mut choseni = 0;
    for i in 0..13 {
        if ((state & (1 << (18-i))) >> (18-i)) == 0 {
            let news = new_state(state, lroll, i);
            let tmp = lookup[news as usize] + score(lroll, i) as f64;
            if tmp > max {
                max = tmp;
                choseni = i;
            }
        }
    }
    (max,choseni)
}

pub fn new_state(state: u32, roll: &[u8;6], cat: usize) -> u32 {
    let mut tmp = state | (1 << (18 - cat));
    let curr_upper = state & (0b11_1111);
    let next_upper = cmp::min(63, curr_upper + upper_score(roll, cat) as u32);
    tmp |= next_upper;
    tmp
}

fn gen_roll_prob(lroll: &[u8; 6], prevroll: &[u8; 6], keep_states: &BTreeMap<[u8; 6], f64>) -> (f64, [u8; 6]) {
    let mut max = 0f64;
    let mut maxroll = [0,0,0,0,0,0];
    for ones in 0..lroll[0]+1 {
        for twos in 0..lroll[1]+1 {
            for threes in 0..lroll[2]+1 {
                for fours in 0..lroll[3]+1 {
                    for fives in 0..lroll[4]+1 {
                        for sixes in 0..lroll[5]+1 {
                            let tmp2 = [ones+prevroll[0],twos+prevroll[1],threes+prevroll[2],fours+prevroll[3],fives+prevroll[4],sixes+prevroll[5]];
                            let tmp = *keep_states.get(&tmp2).expect("asdf");
                            if tmp > max { max = tmp; maxroll = tmp2; }
                        }
                    }
                }
            }
        }
    }
    (max, maxroll)
}

pub fn score(roll: &[u8;6], cat: usize) -> u32 {
    match cat {
        0 ... 5 => {
            return roll[cat] as u32 * ((cat as u32) + 1);
        },
        6 => {
            if roll.iter().fold(false, |a, &b| a || (b >= 3)) {
                let mut tmp = 0;
                for i in 0..6 {
                    tmp += (roll[i] as u32) * (i as u32 + 1)
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
                    tmp += (roll[i] as u32) * (i as u32 + 1)
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
                tmp += (roll[i] as u32) * (i as u32 + 1)
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
    fn test_new_state() {
        assert!(new_state(0b011_1111_1111_1100_0000, &[5,0,0,0,0,0], 0) == 0b111_1111_1111_1100_0101);
        assert!(new_state(0b011_1111_1111_1111_1111, &[5,0,0,0,0,0], 0) == 0b111_1111_1111_1111_1111);
    }

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
