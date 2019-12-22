use aoc2019::result::Result;
use aoc2019::util::read_to_parsed_lines;
use failure::{format_err, Error};

#[derive(Debug)]
enum Instruction {
    DealIntoNewStack,
    Cut { n: i64 },
    DealWithIncrement { n: i64 },
}

impl std::str::FromStr for Instruction {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let t: Vec<_> = s.split(" ").collect();

        let instr = match (t[0], t[1]) {
            ("deal", "into") => Instruction::DealIntoNewStack,
            ("deal", "with") => Instruction::DealWithIncrement { n: t[3].parse()? },
            ("cut", n) => Instruction::Cut { n: n.parse()? },
            _ => return Err(format_err!("Unknown instruction: '{}'", s)),
        };

        Ok(instr)
    }
}

impl Instruction {
    fn deal_into_new_stack(mut stack: Vec<usize>) -> Vec<usize> {
        stack.reverse();
        stack
    }

    fn cut(mut stack: Vec<usize>, n: i64) -> Vec<usize> {
        if n > 0 {
            for _ in 0..n {
                let c = stack.remove(0);
                stack.push(c);
            }
        } else {
            for _ in 0..(-n) {
                let c = stack.pop().unwrap();
                stack.insert(0, c);
            }
        }

        stack
    }

    fn deal_with_increment(mut stack: Vec<usize>, n: i64) -> Vec<usize> {
        let mut new_stack: Vec<Option<usize>> = (0..stack.len()).map(|_| None).collect();
        let n = n as usize;

        let mut i = 0;
        loop {
            let c = stack.remove(0);
            new_stack[i] = Some(c);

            if stack.is_empty() {
                break;
            }

            i = (i + n) % new_stack.len();
            while let Some(_) = new_stack[i] {
                i = (i + 1) % new_stack.len();
            }
        }

        new_stack.into_iter().map(|v| v.unwrap()).collect()
    }

    fn apply(&self, mut stack: Vec<usize>) -> Vec<usize> {
        match self {
            Instruction::DealIntoNewStack => Instruction::deal_into_new_stack(stack),
            Instruction::Cut { n } => Instruction::cut(stack, *n),
            Instruction::DealWithIncrement { n } => Instruction::deal_with_increment(stack, *n),
        }
    }
}

fn main() -> Result<()> {
    let instrs = read_to_parsed_lines("data/day22/input", &|l: &str| l.parse::<Instruction>())?;

    println!("PART ONE");
    let stack_size = 10007;
    let mut stack: Vec<usize> = (0..stack_size).collect();

    for instr in &instrs {
        stack = instr.apply(stack);
    }

    // println!("Stack is {:?}", stack);

    if let Some(i) = stack
        .iter()
        .enumerate()
        .find_map(|(i, v)| if *v == 2019 { Some(i) } else { None })
    {
        println!("2019 is at position {}", i);
    }

    println!("PART TWO");
    let stack_size = 119315717514047;
    let mut stack: Vec<usize> = (0..stack_size).collect();

    for _ in 0..101741582076661usize {
        for instr in &instrs {
            stack = instr.apply(stack);
        }
    }

    if let Some(i) = stack
        .iter()
        .enumerate()
        .find_map(|(i, v)| if *v == 2020 { Some(i) } else { None })
    {
        println!("2020 is at position {}", i);
    }

    Ok(())
}
