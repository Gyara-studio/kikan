use std::collections::HashMap;

pub type UnitScore = u32;

pub trait UnitPart: Sized {
    fn score(&self) -> UnitScore;
}

impl<T> UnitPart for Vec<T>
where
    T: UnitPart,
{
    fn score(&self) -> UnitScore {
        self.iter().map(|unit| unit.score()).sum()
    }
}

impl<U, T> UnitPart for HashMap<U, T>
where
    T: UnitPart,
{
    fn score(&self) -> UnitScore {
        self.iter().map(|(_, unit)| unit.score()).sum()
    }
}
