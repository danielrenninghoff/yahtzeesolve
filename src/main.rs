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

extern crate term;
extern crate yahtzeesolve;

use std::env;
use std::io;
use std::sync::mpsc;
use yahtzeesolve::LookupTable;
use yahtzeesolve::game::generators;
use yahtzeesolve::game::rules;
use yahtzeesolve::game::Game;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("USAGE: {} [generate|play]", &args[0]);
        }
        2 => {
            match &args[1][..] {
                "generate" => {
                    let mut term = term::stdout().unwrap();
                    let (tx, rx) = mpsc::channel();
                    let thread = LookupTable::generate(tx);
                    for i in 0..100 {
                        term.carriage_return().unwrap();
                        write!(term, "Generating probabillity table... {}%", i).unwrap();
                        term.flush().unwrap();
                        rx.recv().unwrap();
                    }
                    let lookup = thread.join().unwrap();
                    lookup.write_to_file("probs.dat").unwrap();
                },
                "play" => {
                    play();
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

fn play() {
    let rollvec = generators::generate_dice_roll_possibilities();
    let dicekeeps = generators::generate_dice_keep_possibilities();
    let x = LookupTable::from_file("probs.dat").unwrap();
    let mut state = Game::new();
    for _ in 0..13 {
        state = calc_round(state, &x, &rollvec, &dicekeeps);
    }
}

fn calc_round(game: Game, lookup: &LookupTable, rollvec: &Vec<[u8; 6]>, dicekeeps: &Vec<[u8; 6]>) -> Game {
    let (keep_1_states, keep_2_states) = yahtzeesolve::precalc_current_round(game, lookup, rollvec, dicekeeps);

    println!("New Round. Please enter your next roll:");
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    let input: u32 = line.trim().parse().unwrap();
    let inp1 = key_conv(input);
    let (_,kroll) = generators::gen_roll_prob(&inp1,&[0,0,0,0,0,0], &keep_1_states);
    println!("{:?}", kroll);
    println!("Please enter your 2nd roll:");
    line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    let input2: u32 = line.trim().parse().unwrap();
    let roll2 = key_conv(input2);
    let (_,kroll) = generators::gen_roll_prob(&roll2,&kroll, &keep_2_states);
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
    let (_,choseni) = generators::choose_best_field(game, &roll2, lookup);
    println!("Mark {}", choseni + 1);
    let scr = rules::score(&roll2, choseni);
    println!("Score: {}", scr);
    game.next_turn(&roll2, choseni)
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
