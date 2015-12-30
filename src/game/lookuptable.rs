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
extern crate num_cpus;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use game::generators;
use game::Game;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::Sender;
use std::sync::mpsc;
use std::sync::Arc;

#[derive(Clone)]
pub struct LookupTable(pub Vec<f64>);

impl LookupTable {
    pub fn from_file(filename: &str) -> io::Result<LookupTable> {
        let mut lookup = vec![0f64; 524288];
        let file = try!(File::open(&Path::new(filename)));
        let mut reader = BufReader::new(file);
        for i in 0..524288 {
            lookup[i] = try!(reader.read_f64::<LittleEndian>());
        }
        Ok(LookupTable(lookup))
    }

    pub fn generate(tx: Sender<()>) -> JoinHandle<LookupTable> {
        thread::spawn(move || {
            let mut lookup = LookupTable(vec![0f64; 524288]);
            let rollvec = Arc::new(generators::generate_dice_roll_possibilities());
            let dicekeeps = Arc::new(generators::generate_dice_keep_possibilities());
            let num_threads = num_cpus::get() as u32;
            let mut progress = 8192;
            for i in (0..8192).rev() {
                if (progress-i) >= 81 {
                    progress = i;
                    tx.send(()).unwrap();
                }
                let (tx2, rx2) = mpsc::channel();

                for j in 0..num_threads {
                    let lookup = lookup.clone();
                    let tx2 = tx2.clone();
                    let rollvec = rollvec.clone();
                    let dicekeeps = dicekeeps.clone();
                    thread::spawn(move || {
                        for k in ((i*64+(j*(64/num_threads)))..(i*64+((j+1)*(64/num_threads)))).rev() {
                            let tmp = generators::gen_start_prob(Game(j), &lookup, &rollvec, &dicekeeps);
                            tx2.send((k, tmp)).unwrap();
                        }
                    });
                }
                for _ in 0..64 {
                    let (num, val) = rx2.recv().unwrap();
                    lookup.set(num, val);
                }
            }
            lookup
        })
    }

    fn set(&mut self, n: u32, value: f64) {
        let &mut LookupTable(ref mut lookup) = self;
        lookup[n as usize] = value;
    }

    pub fn write_to_file(&self, filename: &str) -> io::Result<()> {
        let &LookupTable(ref lookup) = self;
        let file = try!(File::create(&Path::new(filename)));
        let mut writer = BufWriter::new(file);
        for i in 0..524288 {
            try!(writer.write_f64::<LittleEndian>(lookup[i]));
        }
        Ok(())
    }
}
