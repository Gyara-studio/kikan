use crate::{command::Command, layer::Layer};
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub(crate) enum ParseError {
    #[error("invalid command `{0}`")]
    InvalidCommand(char),
}

pub(crate) struct Parser {}

impl Parser {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Layer for Parser {
    type Err = ParseError;
    type Input = char;
    type Output = Command;

    fn update(&mut self, input: Self::Input) -> Result<Self::Output, Self::Err> {
        // see Command's docs for more details
        let out = match input {
            'P' => Command::Port,
            'S' => Command::Starboard,
            'A' => Command::Ahead,
            'N' => Command::Astern,
            ' ' => Command::SE,
            _ => return Err(ParseError::InvalidCommand(input)),
        };
        Ok(out)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Command::*;

    #[test]
    fn basic_test() -> anyhow::Result<()> {
        let input = 'P';
        let mut parser = Parser::new();
        assert_eq!(parser.update(input)?, Port);
        Ok(())
    }

    #[test]
    fn stream_test() -> anyhow::Result<()> {
        let input = "PSAN ";
        let mut parser = Parser::new();
        let res: Result<Vec<Command>, ParseError> = input.chars().map(|ch| parser.update(ch)).collect();
        let res = res?;
        assert_eq!(res, vec![Port, Starboard, Ahead, Astern, SE]);
        Ok(())
    }
}
