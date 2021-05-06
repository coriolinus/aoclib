//! Run with: `cargo test --test parse_2015_day_23`

type Pointer = i32;

/// The registers are named `a` and `b`, and can hold any non-negative integer
#[derive(Debug, parse_display::FromStr)]
#[display(style = "snake_case")]
pub enum Register {
    A,
    B,
}

#[derive(Debug, parse_display::FromStr)]
pub enum Direction {
    #[display("+")]
    Forward,
    #[display("-")]
    Back,
}

#[derive(Debug, parse_display::FromStr)]
#[display("{direction}{distance}")]
#[from_str(regex = r"(?P<direction>.)(?P<distance>\d+)")]
pub struct Offset {
    pub direction: Direction,
    pub distance: Pointer,
}

#[derive(Debug, parse_display::FromStr)]
#[display(style = "snake_case")]
pub enum Instruction {
    /// `hlf r` sets register `r` to half its current value, then continues with the next instruction.
    #[display("{} {0}")]
    Hlf(Register),
    /// `tpl r` sets register `r` to triple its current value, then continues with the next instruction.
    #[display("{} {0}")]
    Tpl(Register),
    /// `inc r` increments register `r`, adding `1` to it, then continues with the next instruction.
    #[display("{} {0}")]
    Inc(Register),
    /// `jmp offset` is a jump; it continues with the instruction `offset` away relative to itself.
    #[display("{} {0}")]
    Jmp(Offset),
    /// `jie r, offset` is like `jmp`, but only jumps if register `r` is even ("jump if even").
    #[display("{} {0}, {1}")]
    Jie(Register, Offset),
    /// `jio r, offset` is like `jmp`, but only jumps if register `r` is 1 ("jump if one", not odd).
    #[display("{} {0}, {1}")]
    Jio(Register, Offset),
}

const EXAMPLE: &str = r#"
inc a
jio a, +2
tpl a
inc a
"#;

#[test]
fn test_example_parses() {
    let insts: Vec<Instruction> = aoclib::input::parse_str(EXAMPLE.trim()).unwrap().collect();
    println!("Instructions: {:#?}", insts);
    assert_eq!(insts.len(), 4);
}
