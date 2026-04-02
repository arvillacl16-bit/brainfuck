use std::path::PathBuf;
use clap::Parser;
use anyhow::{Context, Result};
use std::fs::read_to_string;
use std::alloc::{alloc, dealloc, Layout};
use std::io;
use std::collections::HashMap;

#[derive(Debug, Clone, Parser)]
struct Args {
    file: PathBuf,

    #[arg(short, long, default_value_t = 67108864)]
    block_size: usize,
}

#[derive(Debug)]
struct Memory {
    start: *mut u8,
    layout: Layout,
    size: usize,
    offset: usize,
}

impl Memory {
    fn with_size(size: usize) -> Self {
        unsafe {
            let layout = Layout::array::<u8>(size).unwrap();
            let res = Self {
                start: alloc(layout),
                layout: layout,
                size: size,
                offset: 0,
            };

            for i in 0..size {
                *(res.start.add(i)) = 0;
            }

            res
        }
    }

    fn get(&self) -> &u8 {
        unsafe {
            &*(self.start.add(self.offset))
        }
    }

    fn get_mut(&mut self) -> &mut u8 {
        unsafe {
            &mut *(self.start.add(self.offset))
        }
    }

    fn step_forw(&mut self) {
        self.offset = (self.offset + 1).min(self.size - 1);
    }

    fn step_back(&mut self) {
        if self.offset > 0 {
            self.offset -= 1;
        }
    }

    fn increment(&mut self) {
        *self.get_mut() = self.get_mut().wrapping_add(1);
    }

    fn decrement(&mut self) {
        *self.get_mut() = self.get_mut().wrapping_sub(1);
    }

    fn output(&self) {
        print!("{}", *self.get() as char);
    }

    fn input(&mut self) {
        let mut res = String::new();
        
        io::stdin().read_line(&mut res).unwrap();
        let c = res.trim().chars().next();
    
        *self.get_mut() = c.unwrap() as u8;
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.start, self.layout);
        }
    }
}

fn get_bracket_map(src: &str) -> HashMap<usize, usize> {
    let mut stack = Vec::new();
    let mut bracket_map = HashMap::new();

    for (i, c) in src.chars().enumerate() {
        match c {
            '[' => stack.push(i),
            ']' => {
                let open = stack.pop().expect("Unmatched ]");
                bracket_map.insert(i, open);
                bracket_map.insert(open, i);
            }
            _ => {}
        }
    }

    if !stack.is_empty() {
        panic!("Unmatched [");
    }

    bracket_map
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.file;
    let src = read_to_string(&path)
        .with_context(|| format!("Could not open file {}", path.display()))?;

    let mut memory = Memory::with_size(args.block_size);
    
    let bracket_map = get_bracket_map(&src);

    let mut idx = 0;
    let src_vec: Vec<char> = src.chars().collect();
    while idx < src_vec.len() {
        match src_vec[idx] {
            '>' => memory.step_forw(),
            '<' => memory.step_back(),
            '+' => memory.increment(),
            '-' => memory.decrement(),
            '.' => memory.output(),
            ',' => memory.input(),
            '[' => {
                if *memory.get() == 0 {
                    idx = bracket_map[&idx];
                }
            }
            ']' => {
                if *memory.get() != 0 {
                    idx = bracket_map[&idx];
                }
            }
            _  => {},
        }
        idx += 1;
    }
    println!();
    Ok(())
}
