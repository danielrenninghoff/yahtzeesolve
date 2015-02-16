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

#![feature(core)]
#![feature(env)]
#![feature(io)]
#![feature(path)]
#![feature(std_misc)]

use std::collections::BTreeMap;
use std::iter::AdditiveIterator;
use std::sync::TaskPool;
use std::sync::mpsc;
use std::sync::Arc;
use std::old_io::File;
use std::old_io::BufferedWriter;
use std::old_io::BufferedReader;
use std::old_io::stdio;
use std::env;
use std::old_io::IoResult;

mod state;


fn generate_dice_rolls() -> Vec<[u8; 6]> {
    let mut rolls = vec![];
    for ones in 0..6u8 {
        for twos in 0..6u8 {
            for threes in 0..6u8 {
                for fours in 0..6u8 {
                    for fives in 0..6u8 {
                        for sixes in 0..6u8 {
                            if (ones+twos+threes+fours+fives+sixes) == 5 {
                                rolls.push([ones,twos,threes,fours,fives,sixes]);
                            }
                        }
                    }
                }
            }
        }
    }
    rolls
}

fn generate_dice_keeps() -> Vec<[u8; 6]> {
    let mut keeps = vec![];
    for ones in 0..6u8 {
        for twos in 0..6u8 {
            for threes in 0..6u8 {
                for fours in 0..6u8 {
                    for fives in 0..6u8 {
                        for sixes in 0..6u8 {
                            if (ones+twos+threes+fours+fives+sixes) <= 5 {
                                keeps.push([ones,twos,threes,fours,fives,sixes]);
                            }
                        }
                    }
                }
            }
        }
    }
    keeps
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match &args[] {
        [ref name] => {
            println!("USAGE: {} [generate|play]", name);
        },
        [_, ref one] => {
            match &one[] {
                "generate" => {
                    let mut x: BTreeMap<state::State, f64> = BTreeMap::new();
                    let rollvec = generate_dice_rolls();
                    let dicekeeps = generate_dice_keeps();
                    let rolls = Arc::new(rollvec);
                    let keeps = Arc::new(dicekeeps);
                    let mut start = state::State{ fields: [true;13], upper: 63 };
                    let pool = TaskPool::new(4);
                    let (tx, rx) = mpsc::channel();
                    for i in 0..14 {
                        println!("Generating probabilities for {} empty fields...", i);
                        let x2 = Arc::new(x.clone());
                        let mut cnt = 0;
                        loop {
                            let tx = tx.clone();
                            let start2 = start.clone();
                            let x3 = x2.clone();
                            let rolls2 = rolls.clone();
                            let keeps2 = keeps.clone();
                            pool.execute(move || {
                                let tmp = gen_start_prob(&start2, &*x3, &*rolls2, &*keeps2);
                                tx.send((start2,tmp)).unwrap();
                            });
                            cnt += 1;
                            if calc_next(&mut start) {
                                println!("{:?}",start);
                                break;
                            }
                        }
                        for _ in (0..cnt) {
                            let (e,e2) = rx.recv().unwrap();
                            x.insert(e,e2);
                        }
                    }
                    write_state_file(&x).unwrap();
                },
                "play" => {
                    let rollvec = generate_dice_rolls();
                    let dicekeeps = generate_dice_keeps();
                    let x = read_state_file().unwrap();
                    let mut state = state::State{ fields: [false;13], upper: 0 };
                    for _ in 0..13 {
                        state = calc_round(&state, &x, &rollvec, &dicekeeps);
                    }
                }
                _ => {

                }
            }

        },
        _ => {
            println!("USAGE: yahtzee [generate|play]");
        }
    }
}

fn write_state_file(map: &BTreeMap<state::State, f64>) -> IoResult<()> {
    let file = try!(File::create(&Path::new("probs.dat")));
    let mut writer = BufferedWriter::new(file);
    for (st, f) in map {
        for e in &st.fields {
            if *e {
                try!(writer.write_u8(1));
            }
            else {
                try!(writer.write_u8(0));
            }
        }
        try!(writer.write_u8(st.upper));
        try!(writer.write_le_f64(*f));
    }
    Ok(())
}

fn read_state_file() -> IoResult<BTreeMap<state::State, f64>> {
    let mut map: BTreeMap<state::State, f64> = BTreeMap::new();
    let file = try!(File::open(&Path::new("probs.dat")));
    let mut reader = BufferedReader::new(file);
    let mut state = state::State{ fields: [true;13], upper: 63 };
    'bigloop: loop {
        for i in 0..13us {
            match reader.read_u8() {
                Ok(v) => {
                    if v == 0u8 {
                        state.fields[i] = false;
                    }
                    else {
                        state.fields[i] = true;
                    }
                },
                Err(_) => {
                    break 'bigloop;
                }
            }
        }
        state.upper = try!(reader.read_u8());
        let prob = try!(reader.read_le_f64());
        map.insert(state.clone(), prob);
    }
    Ok(map)
}

fn calc_round(state: &state::State, lookup: &BTreeMap<state::State, f64>, rollvec: &Vec<[u8; 6]>, dicekeeps: &Vec<[u8; 6]>) -> state::State {
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
    let input: u32 = stdio::stdin().read_line().unwrap().trim().parse().unwrap();
    let inp1 = key_conv(input);
    let (_,kroll) = gen_roll_prob(&inp1,&[0,0,0,0,0,0], &keep_1_states);
    println!("{:?}", kroll);
    println!("Please enter your 2nd roll:");
    let input2: u32 = stdio::stdin().read_line().unwrap().trim().parse().unwrap();
    let roll2 = key_conv(input2);
    let (_,kroll) = gen_roll_prob(&roll2,&kroll, &keep_2_states);
    println!("{:?}", kroll);
    println!("Please enter your 3rd roll:");
    let input2: u32 = stdio::stdin().read_line().unwrap().trim().parse().unwrap();
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
    state::new_state(state, &roll2, choseni)
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

fn calc_next(state: &mut state::State) -> bool {
    match state.upper {
        0 => state.upper = 63,
        _ => {
            state.upper -= 1;
            return false;
        }
    }

    let n = state.fields.iter().fold(0, |a, &b| {
        if !b {
            return a + 1;
        }
        else {
            return a;
        }
    });

    let mut n3 = n;

    if n == 0 {
        state.fields[0] = false;
        return true;
    }

    if n == 13 {
        return true;
    }

    for i in 0..13us {
        if (!state.fields[i]) && n3 == 1 {
            if i != 12 {
                state.fields[i] = true;
                state.fields[i+1] = false;
                return false;
            }
            else {
                let mut n2 = 1;
                for j in (0..12us).rev() {
                    if !state.fields[j] && state.fields[j+1] {
                        state.fields[j] = true;
                        for k in (0us..n2+1) {
                            state.fields[(12-k)] = true;
                        }
                        for k in (0us..n2+1) {
                            state.fields[(j+1+k)] = false;
                        }
                        state.fields[j+1] = false;
                        //println!("asdasd7777 {:?}", state);
                        return false;
                    }
                    else if !state.fields[j] {
                        n2 += 1;
                    }
                }
                for j in 0..13us {
                    if j <= n {
                        state.fields[j] = false;
                    }
                    else {
                        state.fields[j] = true;
                    }
                }
                //println!("asdasd999 {:?}", state);
                return true;
            }
        }
        else if !state.fields[i] {
            n3 -= 1;
        }
    }
    println!("ERROR SWASASASS");
    return false;
}

fn gen_start_prob(state: &state::State, lookup: &BTreeMap<state::State, f64>, rollvec: &Vec<[u8; 6]>, dicekeeps: &Vec<[u8; 6]>) -> f64 {
    if state.fields.iter().fold(true,|a, &b| a && b) {
        match state.upper {
            63 => return 35f64,
            _  => return 0f64,
        }
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
        for twos in lroll[1]..6 {
            for threes in lroll[2]..6 {
                for fours in lroll[3]..6 {
                    for fives in lroll[4]..6 {
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
    (sum / cnt)
}

fn gen_end_prob(state: &state::State, lroll: &[u8; 6], lookup: &BTreeMap<state::State, f64>) -> (f64, usize) {
    let mut max = 0f64;
    let mut choseni = 0;
    for i in 0..13us {
        if state.fields[i] == false {
            let tmp;
            let news = state::new_state(state, lroll, i);
            match lookup.get(&news) {
                Some(x) => {
                    //println!("score: {} {}", score(lroll, i), *x);
                    tmp = *x + score(lroll, i);
                }
                None => {
                    println!("{:?} {:?}", state, news);
                    panic!();
                }
            }
            if tmp > max { max = tmp; choseni = i; }
        }
    }
    (max,choseni)
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

fn score(roll: &[u8;6], cat: usize) -> f64 {
    match cat {
        0 ... 5 => {
            return (roll[cat] * ((cat as u8) + 1)) as f64;
        },
        6 => {
            if roll.iter().fold(false, |a, &b| a || (b >= 3)) {
                return (0..6us).map(|i| roll[i] * ((i as u8)+1)).sum() as f64;
            }
            else {
                return 0f64;
            }
        }
        7 => {
            if roll.iter().fold(false, |a, &b| a || (b >= 4)) {
                return (0..6us).map(|i| roll[i] * ((i as u8)+1)).sum() as f64;
            }
            else {
                return 0f64;
            }
        }
        8 => {
            if roll.contains(&3) && roll.contains(&2) {
                return 25f64;
            }
            else {
                return 0f64;
            }
        }
        9 => {
            if    (roll[0] >= 1 && roll[1] >= 1 && roll[2] >= 1 && roll[3] >= 1)
               || (roll[1] >= 1 && roll[2] >= 1 && roll[3] >= 1 && roll[4] >= 1)
               || (roll[2] >= 1 && roll[3] >= 1 && roll[4] >= 1 && roll[5] >= 1) {
                   return 30f64;
               }
            else {
                return 0f64;
            }
        }
        10 => {
            if    (roll[0] == 1 && roll[1] == 1 && roll[2] == 1 && roll[3] == 1 && roll[4] == 1)
               || (roll[1] == 1 && roll[2] == 1 && roll[3] == 1 && roll[4] == 1 && roll[5] == 1) {
                   return 40f64;
               }
            else {
                return 0f64;
            }
        }
        11 => {
            if roll.contains(&5) {
                return 50f64;
            }
            else {
                return 0f64;
            }
        }
        12 => {
            return (0..6us).map(|i| roll[i] * ((i as u8)+1)).sum() as f64;
        }
        _ => {
            return 0f64;
        }
    };
}
