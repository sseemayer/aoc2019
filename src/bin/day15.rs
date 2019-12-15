use aoc2019::intcode::{parse_program, IntCodeResult, State};
use aoc2019::result::{format_err, Error, Result};
use aoc2019::util::read_to_string;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct Position {
    i: i64,
    j: i64,
}

impl Position {
    const ZERO: Position = Position { i: 0, j: 0 };
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "({}, {})", self.j, self.i)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Tile {
    Unknown,
    Empty,
    Wall,
    OxygenSystem,
}

impl Tile {
    fn from_output(o: i64) -> Result<Self> {
        match o {
            0 => Ok(Tile::Wall),
            1 => Ok(Tile::Empty),
            2 => Ok(Tile::OxygenSystem),

            _ => Err(format_err!("Output is not a tile: {}", o)),
        }
    }
}

#[derive(Copy, Clone)]
enum Direction {
    North,
    South,
    West,
    East,
}

impl std::fmt::Debug for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let c = match self {
            Direction::North => "🠉",
            Direction::South => "🠋",
            Direction::West => "🠈 ",
            Direction::East => "🠊 ",
        };
        write!(f, "{}", c)
    }
}

impl Direction {
    const ALL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    fn to_input(&self) -> i64 {
        match self {
            Direction::North => 1,
            Direction::South => 2,
            Direction::West => 3,
            Direction::East => 4,
        }
    }

    fn to_ofs(&self) -> (i64, i64) {
        match self {
            Direction::North => (-1, 0),
            Direction::South => (1, 0),
            Direction::West => (0, -1),
            Direction::East => (0, 1),
        }
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let c = match self {
            Tile::Unknown => "░",
            Tile::Empty => " ",
            Tile::Wall => "█",
            Tile::OxygenSystem => "O",
        };
        write!(f, "{}", c)
    }
}

enum BfsResult {
    Found { pos: Position, path: Vec<Direction> },
    NotFound { max_path_len: usize },
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
                let t = self.tiles.get(&Position { i, j }).unwrap_or(&Tile::Unknown);

                write!(f, "{}", t)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl Board {
    fn new() -> Self {
        let mut tiles = HashMap::new();
        tiles.insert(Position::ZERO, Tile::Empty);
        Board { tiles }
    }

    fn get(&self, pos: &Position) -> &Tile {
        self.tiles.get(pos).unwrap_or(&Tile::Unknown)
    }

    fn set(&mut self, pos: &Position, tile: Tile) {
        self.tiles.insert(pos.clone(), tile);
    }

    fn search_bfs(&self, start_pos: &Position, target: &Tile) -> BfsResult {
        // this is breadth first search for a tile of interest, given the current knowledge
        let mut queue = vec![(start_pos.to_owned(), Vec::new())];
        let mut seen = HashSet::new();

        let mut max_path_len = 0;

        seen.insert(start_pos.clone());

        while !queue.is_empty() {
            let (cur, history) = queue.remove(0);
            seen.insert(cur.clone());
            let tile = self.get(&cur);

            if tile == target {
                return BfsResult::Found {
                    pos: cur,
                    path: history,
                };
            } else if tile == &Tile::Wall {
                // don't extend search from this tile
                continue;
            }

            for dir in Direction::ALL.iter() {
                let (i_ofs, j_ofs) = dir.to_ofs();

                let cand_pos = Position {
                    i: cur.i + i_ofs,
                    j: cur.j + j_ofs,
                };

                if !seen.contains(&cand_pos) {
                    let mut new_history: Vec<Direction> = history.iter().map(|h| *h).collect();
                    new_history.push(*dir);

                    if new_history.len() > max_path_len {
                        max_path_len = new_history.len();
                    }

                    queue.push((cand_pos, new_history));
                }
            }
        }

        BfsResult::NotFound { max_path_len }
    }

    fn run_discovery(&mut self, program: &Vec<i64>) -> Result<()> {
        // fill up knowledge of the playfield
        while let BfsResult::Found { pos, path } = self.search_bfs(&Position::ZERO, &Tile::Unknown)
        {
            let n_steps = path.len();
            let mut state = State {
                memory: program.clone(),
                ic: 0,
                inputs: path.iter().map(|d| d.to_input()).collect(),
                relative_base: 0,
            };

            let mut outputs = Vec::new();
            for _ in 0..n_steps {
                state.run(&mut outputs)?;
            }

            //println!(
            //    "Target {}: Path {:?} gives outputs {:?}",
            //    pos, path, outputs
            //);

            let last_output = outputs.pop().unwrap();
            let tile = Tile::from_output(last_output)?;

            self.set(&pos, tile);

            println!("\n\n{}", self);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let program = parse_program(&read_to_string("data/day15/input")?)?;

    println!("Program: {:?}", program);

    let mut board = Board::new();
    board.run_discovery(&program)?;

    println!("BOARD:\n{}", board);

    if let BfsResult::Found { pos: o2s_pos, path } =
        board.search_bfs(&Position::ZERO, &Tile::OxygenSystem)
    {
        println!("Path: {:?} ({} steps)", path, path.len());

        if let BfsResult::NotFound { max_path_len } = board.search_bfs(&o2s_pos, &Tile::Unknown) {
            println!("Flooding time: {} minutes", max_path_len - 1);
        }
    }

    Ok(())
}
