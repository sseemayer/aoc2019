use failure::{format_err, Error};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct State {
    memory: Vec<i64>,
    ic: usize,
    inputs: Vec<i64>,
    relative_base: i64,
}

enum IntCodeResult {
    Input,
    Output,
    Halt,
}

impl State {
    fn get_memory(&mut self, address: usize) -> i64 {
        while self.memory.len() <= address {
            self.memory.push(0);
        }

        self.memory[address]
    }

    fn set_memory(&mut self, address: usize, value: i64) {
        while self.memory.len() <= address {
            self.memory.push(0);
        }

        self.memory[address] = value;
    }

    fn get_address(&mut self, parmodes: &Vec<u8>, ofs: usize) -> Result<usize> {
        let pm = parmodes.get(ofs - 1).unwrap_or(&0);
        let pv = self.memory[self.ic + ofs];

        let addr = match pm {
            0 => {
                // position mode
                pv as usize
            }
            2 => {
                // relative mode
                (self.relative_base + pv) as usize
            }
            _ => return Err(format_err!("Invalid parameter mode at {}: {}", ofs, pm)),
        };

        Ok(addr)
    }

    fn get_parameter(&mut self, parmodes: &Vec<u8>, ofs: usize) -> Result<i64> {
        let pm = parmodes.get(ofs - 1).unwrap_or(&0);
        let pv = self.memory[self.ic + ofs];

        let val = match pm {
            0 => {
                // position mode
                self.get_memory(pv as usize)
            }
            1 => {
                // immediate mode
                pv
            }
            2 => {
                // relative mode
                self.get_memory((self.relative_base + pv) as usize)
            }
            _ => return Err(format_err!("Invalid parameter mode at {}: {}", ofs, pm)),
        };

        Ok(val)
    }

    fn get_opcode_and_parmode(&self) -> (i64, Vec<u8>) {
        let mut v = self.memory[self.ic];

        let opcode = v % 100;
        v /= 100;

        let mut parmodes = Vec::new();

        while v > 0 {
            let pm = v % 10;
            parmodes.push(pm as u8);
            v /= 10;
        }

        return (opcode, parmodes);
    }

    fn run(&mut self, outputs: &mut Vec<i64>) -> Result<IntCodeResult> {
        loop {
            let (opcode, parmodes) = self.get_opcode_and_parmode();

            match opcode {
                1 => {
                    // add
                    let a = self.get_parameter(&parmodes, 1)?;
                    let b = self.get_parameter(&parmodes, 2)?;
                    let pos_store = self.get_address(&parmodes, 3)?;

                    self.set_memory(pos_store, a + b);
                    self.ic += 4;
                }
                2 => {
                    // mul
                    let a = self.get_parameter(&parmodes, 1)?;
                    let b = self.get_parameter(&parmodes, 2)?;
                    let pos_store = self.get_address(&parmodes, 3)?;

                    self.set_memory(pos_store, a * b);
                    self.ic += 4;
                }
                3 => {
                    // input
                    if self.inputs.len() > 0 {
                        let pos_store = self.get_address(&parmodes, 1)? as usize;
                        let input_val = self.inputs.remove(0);
                        self.set_memory(pos_store, input_val);
                        self.ic += 2;
                    } else {
                        return Ok(IntCodeResult::Input);
                    }
                }
                4 => {
                    // output
                    let v = self.get_parameter(&parmodes, 1)?;
                    outputs.push(v);
                    self.ic += 2;
                    return Ok(IntCodeResult::Output);
                }
                5 => {
                    // jump-if-true
                    let a = self.get_parameter(&parmodes, 1)?;
                    let b = self.get_parameter(&parmodes, 2)?;

                    self.ic = if a != 0 { b as usize } else { self.ic + 3 };
                }
                6 => {
                    // jump-if false
                    let a = self.get_parameter(&parmodes, 1)?;
                    let b = self.get_parameter(&parmodes, 2)?;

                    self.ic = if a == 0 { b as usize } else { self.ic + 3 };
                }
                7 => {
                    // less-than
                    let a = self.get_parameter(&parmodes, 1)?;
                    let b = self.get_parameter(&parmodes, 2)?;
                    let pos_store = self.get_address(&parmodes, 3)?;

                    self.set_memory(pos_store, if a < b { 1 } else { 0 });
                    self.ic += 4;
                }
                8 => {
                    // equals
                    let a = self.get_parameter(&parmodes, 1)?;
                    let b = self.get_parameter(&parmodes, 2)?;
                    let pos_store = self.get_address(&parmodes, 3)?;

                    self.set_memory(pos_store, if a == b { 1 } else { 0 });
                    self.ic += 4;
                }
                9 => {
                    // shift relative base
                    let a = self.get_parameter(&parmodes, 1)?;
                    self.relative_base += a;
                    self.ic += 2;
                }
                99 => {
                    // halt
                    return Ok(IntCodeResult::Halt);
                }
                _ => return Err(format_err!("Invalid opcode: {}", opcode)),
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct Position {
    i: i64,
    j: i64,
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "({}, {})", self.j, self.i)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Tile {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl std::convert::TryFrom<i64> for Tile {
    type Error = Error;
    fn try_from(v: i64) -> Result<Self> {
        match v {
            0 => Ok(Tile::Empty),
            1 => Ok(Tile::Wall),
            2 => Ok(Tile::Block),
            3 => Ok(Tile::Paddle),
            4 => Ok(Tile::Ball),
            _ => Err(format_err!("Invalid tile: {}", v)),
        }
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let c = match self {
            Tile::Empty => ".",
            Tile::Wall => "#",
            Tile::Block => "X",
            Tile::Paddle => "=",
            Tile::Ball => "o",
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug)]
struct Board {
    tiles: HashMap<Position, Tile>,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        // find maximum drawing coords
        let mut i_min = 0;
        let mut i_max = 0;
        let mut j_min = 0;
        let mut j_max = 0;
        for pos in self.tiles.keys() {
            if pos.i < i_min {
                i_min = pos.i;
            }
            if pos.i > i_max {
                i_max = pos.i;
            }
            if pos.j < j_min {
                j_min = pos.j;
            }
            if pos.j > j_max {
                j_max = pos.j;
            }
        }

        for i in i_min..=i_max {
            for j in j_min..=j_max {
                let t = self.tiles.get(&Position { i, j }).unwrap_or(&Tile::Empty);

                write!(f, "{}", t)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl Board {
    fn new() -> Self {
        Board {
            tiles: HashMap::new(),
        }
    }

    fn get(&self, pos: &Position) -> &Tile {
        self.tiles.get(pos).unwrap_or(&Tile::Empty)
    }

    fn set(&mut self, pos: &Position, tile: Tile) {
        self.tiles.insert(pos.clone(), tile);
    }

    fn run(&mut self, program: &Vec<i64>) -> Result<()> {
        let inputs = Vec::new();
        let mut state = State {
            memory: program.clone(),
            ic: 0,
            inputs,
            relative_base: 0,
        };

        // run to completion
        let mut outputs = Vec::new();
        loop {
            match state.run(&mut outputs)? {
                IntCodeResult::Halt => break,
                IntCodeResult::Input => {
                    let ball_pos = self.get_pos_of(Tile::Ball).unwrap();
                    let paddle_pos = self.get_pos_of(Tile::Paddle).unwrap();

                    let delta_j = ball_pos.j - paddle_pos.j;

                    let v = if delta_j < 0 {
                        -1
                    } else if delta_j == 0 {
                        0
                    } else {
                        1
                    };
                    state.inputs.push(v);
                    println!("{}\nINPUT: {}", self, v);
                }
                IntCodeResult::Output => {
                    if outputs.len() >= 3 {
                        let j = outputs.remove(0);
                        let i = outputs.remove(0);
                        let t = outputs.remove(0);

                        if (i == 0 && j == -1) {
                            println!("NEW SCORE: {}", t);
                        } else {
                            let pos = Position { i, j };
                            let tile = Tile::try_from(t)?;

                            self.set(&pos, tile);
                        }

                        //println!("Mov {}, direction {:?}", self.robot_pos, self.robot_dir);
                    }
                }
            }
        }

        Ok(())
    }

    fn count(&self) -> HashMap<&Tile, usize> {
        let mut out = HashMap::new();

        for tile in self.tiles.values() {
            out.entry(tile).and_modify(|c| *c += 1).or_insert(1);
        }

        out
    }

    fn get_pos_of(&self, v: Tile) -> Option<Position> {
        for (pos, tile) in self.tiles.iter() {
            if *tile == v {
                return Some(*pos);
            }
        }
        None
    }
}

fn main() -> Result<()> {
    // /*
    let mut f = File::open("data/day13/input")?;

    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    let buf = String::from_utf8(buf)?;
    // */
    let mut program: Vec<i64> = buf
        .trim()
        .split(",")
        .map(|v| v.parse().map_err(|e: std::num::ParseIntError| e.into()))
        .collect::<Result<Vec<i64>>>()?;

    println!("FIRST RUN");
    let mut board = Board::new();

    board.run(&program)?;

    println!("{}", board);

    println!("Counts: {:#?}", board.count());

    println!("SECOND RUN");
    program[0] = 2;

    board.run(&program)?;

    Ok(())
}
