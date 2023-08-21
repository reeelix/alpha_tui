use std::fmt::Display;

use crate::runtime::error_handling::{RuntimeErrorType, CalcError};

/// A single accumulator, represents "Akkumulator/Alpha" from SysInf lecture.
#[derive(Debug, Clone, PartialEq)]
pub struct Accumulator {
    /// Used to identify accumulator
    pub id: usize,
    /// The data stored in the Accumulator
    pub data: Option<i32>,
}

impl Accumulator {
    /// Creates a new accumulator
    pub fn new(id: usize) -> Self {
        Self { id, data: None }
    }
}

impl Display for Accumulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.data {
            Some(d) => write!(f, "{:2}: {}", self.id, d),
            None => write!(f, "{:2}: None", self.id),
        }
    }
}

/// Representation of a single memory cell.
/// The term memory cell is equal to "Speicherzelle" in the SysInf lecture.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCell {
    pub label: String,
    pub data: Option<i32>,
}

impl MemoryCell {
    /// Creates a new register
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            data: None,
        }
    }
}

impl Display for MemoryCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.data {
            Some(d) => write!(f, "{:2}: {}", self.label, d),
            None => write!(f, "{:2}: None", self.label),
        }
    }
}

/// Different ways of paring two values
#[derive(Debug, PartialEq, Clone)]
pub enum Comparison {
    Less,
    LessOrEqual,
    Equal,
    NotEqual,
    MoreOrEqual,
    More,
}

impl Comparison {
    /// Compares two values with the selected method of comparison.
    pub fn cmp(&self, x: i32, y: i32) -> bool {
        match self {
            Self::Less => x < y,
            Self::LessOrEqual => x <= y,
            Self::Equal => x == y,
            Self::NotEqual => x != y,
            Self::MoreOrEqual => x >= y,
            Self::More => x > y,
        }
    }
}

impl TryFrom<&str> for Comparison {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<" => Ok(Self::Less),
            "<=" => Ok(Self::LessOrEqual),
            "=<" => Ok(Self::LessOrEqual),
            "=" => Ok(Self::Equal),
            "==" => Ok(Self::Equal),
            "!=" => Ok(Self::NotEqual),
            ">=" => Ok(Self::MoreOrEqual),
            "=>" => Ok(Self::MoreOrEqual),
            ">" => Ok(Self::More),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operation {
    pub fn calc(&self, x: i32, y: i32) -> Result<i32, RuntimeErrorType> {
        match self {
            Self::Add => {
                match x.checked_add(y) {
                    Some(v) => Ok(v),
                    None => Err(RuntimeErrorType::IllegalCalculation { cause: CalcError::AttemptToOverflow("add".to_string(), "Addition".to_string()) })
                }
            },
            Self::Sub => {
                match x.checked_sub(y) {
                    Some(v) => Ok(v),
                    None => Err(RuntimeErrorType::IllegalCalculation { cause: CalcError::AttemptToOverflow("subtract".to_string(), "Subtraction".to_string()) })
                }
            },
            Self::Mul => {
                match x.checked_mul(y) {
                    Some(v) => Ok(v),
                    None => Err(RuntimeErrorType::IllegalCalculation { cause: CalcError::AttemptToOverflow("multiply".to_string(), "Multiplication".to_string()) })
                }
            },
            Self::Div => {
                if x != y {
                    match x.checked_div(y) {
                        Some(v) => Ok(v),
                        None => Err(RuntimeErrorType::IllegalCalculation { cause: CalcError::AttemptToOverflow("divide".to_string(), "Division".to_string()) })
                    }
                } else {
                    Err(RuntimeErrorType::IllegalCalculation { cause: CalcError::AttemptToDivideByZero() })
                }
            },
        }
    }
}

impl TryFrom<&str> for Operation {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Operation::Add),
            "-" => Ok(Operation::Sub),
            "*" => Ok(Operation::Mul),
            "/" => Ok(Operation::Div),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::base::{Comparison, MemoryCell, Operation};

    use super::Accumulator;

    #[test]
    fn test_accumultor_display() {
        let mut acc = Accumulator::new(0);
        acc.data = Some(5);
        assert_eq!(format!("{}", acc), " 0: 5");
        acc.data = None;
        assert_eq!(format!("{}", acc), " 0: None");
    }

    #[test]
    fn test_memory_cell_display() {
        let mut acc = MemoryCell::new("a");
        acc.data = Some(5);
        assert_eq!(format!("{}", acc), "a : 5");
        acc.data = None;
        assert_eq!(format!("{}", acc), "a : None");
    }

    #[test]
    fn test_comparison() {
        assert!(Comparison::Less.cmp(5, 10));
        assert!(Comparison::LessOrEqual.cmp(5, 10));
        assert!(Comparison::LessOrEqual.cmp(5, 5));
        assert!(Comparison::Equal.cmp(5, 5));
        assert!(Comparison::NotEqual.cmp(5, 6));
        assert!(!Comparison::NotEqual.cmp(6, 6));
        assert!(Comparison::MoreOrEqual.cmp(5, 5));
        assert!(Comparison::MoreOrEqual.cmp(10, 5));
        assert!(Comparison::More.cmp(10, 5));
    }

    #[test]
    fn test_comparison_try_from_str() {
        assert_eq!(Comparison::try_from("<"), Ok(Comparison::Less));
        assert_eq!(Comparison::try_from("<="), Ok(Comparison::LessOrEqual));
        assert_eq!(Comparison::try_from("=<"), Ok(Comparison::LessOrEqual));
        assert_eq!(Comparison::try_from("="), Ok(Comparison::Equal));
        assert_eq!(Comparison::try_from("=="), Ok(Comparison::Equal));
        assert_eq!(Comparison::try_from("!="), Ok(Comparison::NotEqual));
        assert_eq!(Comparison::try_from(">="), Ok(Comparison::MoreOrEqual));
        assert_eq!(Comparison::try_from("=>"), Ok(Comparison::MoreOrEqual));
        assert_eq!(Comparison::try_from(">"), Ok(Comparison::More));
    }

    #[test]
    fn test_operation() {
        assert_eq!(Operation::Add.calc(20, 5).unwrap(), 25);
        assert_eq!(Operation::Sub.calc(20, 5).unwrap(), 15);
        assert_eq!(Operation::Mul.calc(20, 5).unwrap(), 100);
        assert_eq!(Operation::Div.calc(20, 5).unwrap(), 4);
    }

    #[test]
    fn test_operation_try_from_str() {
        assert_eq!(Operation::try_from("+"), Ok(Operation::Add));
        assert_eq!(Operation::try_from("-"), Ok(Operation::Sub));
        assert_eq!(Operation::try_from("*"), Ok(Operation::Mul));
        assert_eq!(Operation::try_from("/"), Ok(Operation::Div));
        assert_eq!(Operation::try_from("P"), Err(()));
    }
}
