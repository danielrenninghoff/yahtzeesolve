use std::collections::BTreeMap;
use std::num::Float;
use std::iter::AdditiveIterator;
use std::sync::TaskPool;
use std::sync::mpsc;
use std::sync::Arc;
use std::io::File;
use std::io::BufferedWriter;
use std::io::BufferedReader;
use std::io;
use std::os;

mod state;


fn generate_dice_rolls(rolls: &mut Vec<[u8; 6]>) {
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
}

fn generate_dice_keeps(rolls: &mut Vec<[u8; 6]>) {
    for ones in 0..6u8 {
        for twos in 0..6u8 {
            for threes in 0..6u8 {
                for fours in 0..6u8 {
                    for fives in 0..6u8 {
                        for sixes in 0..6u8 {
                            if (ones+twos+threes+fours+fives+sixes) <= 5 {
                                rolls.push([ones,twos,threes,fours,fives,sixes]);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let mut x: BTreeMap<state::State, f64> = BTreeMap::new();

    let args = os::args();
    match args.as_slice() {
        [ref name] => {
            println!("USAGE: {} [generate|play]", name);
        },
        [_, ref one] => {
            match one.as_slice() {
                "generate" => {
                    let mut rollvec = vec![];
                    generate_dice_rolls(&mut rollvec);
                    let mut dicekeeps = vec![];
                    generate_dice_keeps(&mut dicekeeps);
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
                    let file = File::create(&Path::new("probs.dat")).unwrap();
                    let mut writer = BufferedWriter::new(file);
                    for (st, f) in x.iter() {
                        for e in st.fields.iter() {
                            if *e {
                                writer.write_u8(1);
                            }
                            else {
                                writer.write_u8(0);
                            }
                        }
                        writer.write_u8(st.upper);
                        writer.write_le_f64(*f);
                    }
                },
                "play" => {
                    let mut rollvec = vec![];
                    generate_dice_rolls(&mut rollvec);
                    let mut dicekeeps = vec![];
                    generate_dice_keeps(&mut dicekeeps);
                    readStateFile(&mut x);
                    let mut state = state::State{ fields: [false;13], upper: 0 };
                    for i in 0..13 {
                        state = calc_round(&mut state, &x, &rollvec, &dicekeeps);
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

fn readStateFile(map: &mut BTreeMap<state::State, f64>) {
    let file = File::open(&Path::new("probs.dat")).unwrap();
    let mut reader = BufferedReader::new(file);
    let mut state = state::State{ fields: [true;13], upper: 63 };
'bigloop: loop {
        for i in 0..13 {
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
        state.upper = reader.read_u8().unwrap();
        let prob = reader.read_le_f64().unwrap();
        map.insert(state.clone(), prob);
    }
}

fn calc_round(state: &state::State, lookup: &BTreeMap<state::State, f64>, rollvec: &Vec<[u8; 6]>, dicekeeps: &Vec<[u8; 6]>) -> state::State {
    let mut end_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec.iter() {
        let (tmp,_) = gen_end_prob(state, roll, lookup);
        end_states.insert(*roll, tmp);
    }

    let mut keep_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps.iter() {
        keep_2_states.insert(*keep, gen_keep_prob(keep, &end_states));
    }

    let mut roll_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec.iter() {
        let (tmp,_) = gen_roll_prob(roll,&[0,0,0,0,0,0], &keep_2_states);
        roll_2_states.insert(*roll, tmp);
    }

    let mut keep_1_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps.iter() {
        keep_1_states.insert(*keep, gen_keep_prob(keep, &roll_2_states));
    }

    println!("New Round. Please enter your next roll:");
    let input: u32 = io::stdin().read_line().unwrap().trim().parse().unwrap();
    let inp1 = key_conv(input);
    let (_,kroll) = gen_roll_prob(&inp1,&[0,0,0,0,0,0], &keep_1_states);
    println!("{:?}", kroll);
    println!("Please enter your 2nd roll:");
    let input2: u32 = io::stdin().read_line().unwrap().trim().parse().unwrap();
    let roll2 = key_conv(input2);
    let (_,kroll) = gen_roll_prob(&roll2,&kroll, &keep_2_states);
    println!("{:?}", kroll);
    println!("Please enter your 3rd roll:");
    let input2: u32 = io::stdin().read_line().unwrap().trim().parse().unwrap();
    let mut roll2 = key_conv(input2);
    roll2[0] += kroll[0];
    roll2[1] += kroll[1];
    roll2[2] += kroll[2];
    roll2[3] += kroll[3];
    roll2[4] += kroll[4];
    roll2[5] += kroll[5];
    let (_,choseni) = gen_end_prob(state, &roll2, lookup);
    println!("Mark {}", choseni + 1);
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

    for i in 0..13 {
        if (!state.fields[i]) && n3 == 1 {
            if i != 12 {
                state.fields[i] = true;
                state.fields[i+1] = false;
                return false;
            }
            else {
                let mut n2 = 1;
                for j in (0..12).rev() {
                    if !state.fields[j] && state.fields[j+1] {
                        state.fields[j] = true;
                        for k in (0..n2+1) {
                            state.fields[(12-k)] = true;
                        }
                        for k in (0..n2+1) {
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
                for j in 0..13 {
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
    for roll in rollvec.iter() {
        let (tmp,_) = gen_end_prob(state, roll, lookup);
        end_states.insert(*roll, tmp);
    }

    let mut keep_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps.iter() {
        keep_2_states.insert(*keep, gen_keep_prob(keep, &end_states));
    }

    let mut roll_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec.iter() {
        let (tmp,_) = gen_roll_prob(roll,&[0,0,0,0,0,0], &keep_2_states);
        roll_2_states.insert(*roll, tmp);
    }

    let mut keep_1_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps.iter() {
        keep_1_states.insert(*keep, gen_keep_prob(keep, &roll_2_states));
    }

    let mut roll_1_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec.iter() {
        let (tmp,_) = gen_roll_prob(roll,&[0,0,0,0,0,0], &keep_1_states);
        roll_1_states.insert(*roll, tmp);
    }

    let mut sum = 0f64;
    let mut cnt = 0f64;
    for roll in rollvec.iter() {
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
                return (0..6).map(|i| roll[i] * ((i as u8)+1)).sum() as f64;
            }
            else {
                return 0f64;
            }
        }
        7 => {
            if roll.iter().fold(false, |a, &b| a || (b >= 4)) {
                return (0..6).map(|i| roll[i] * ((i as u8)+1)).sum() as f64;
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
            return (0..6).map(|i| roll[i] * ((i as u8)+1)).sum() as f64;
        }
        12 => {
            if roll.iter().fold(false, |a, &b| a || (b == 5)) {
                return 50f64;
            }
            else {
                return 0f64;
            }
        }
        _ => {
            return 0f64;
        }
    };
}
