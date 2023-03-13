use crate::{MAX_COST, MAX_ID};
use derive_more::{AddAssign, SubAssign};
use std::ops::{Index, IndexMut};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Id(u8);

#[derive(
    Debug, Default, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, AddAssign, SubAssign,
)]
pub struct Energy(u8);

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Card(u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CardIdentity {
    Full(Card),
    Cost(Energy),
    None,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct TurnNumber(u8);

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Turn {
    pub number: TurnNumber,
    pub energy: Energy,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Deck<T, const N: usize>([T; N]);

#[derive(Debug, Copy, Clone)]
pub struct CostProfile<const N: usize>([u8; MAX_COST as usize]);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct EnergyProfile<const M: usize>([u8; M]);

#[derive(Debug, Copy, Clone)]
pub struct TurnProfile<const N: usize>([Turn; N]);

impl Card {
    pub fn new(id: u8, cost: u8) -> Self {
        if id >= MAX_ID {
            panic!(
                "Card id {} is greater or equal to max card id ({})",
                id, MAX_ID
            );
        }
        if cost > MAX_COST {
            panic!(
                "Card cost {} is greater than max card cost ({})",
                cost, MAX_COST
            );
        }
        Card((id << 4) | cost)
    }

    pub fn id(self) -> Id {
        Id(self.0 >> 4)
    }
}

impl<T: Copy, const N: usize> Deck<T, N> {
    pub fn card_iter(&self) -> impl Iterator<Item = T> {
        self.0.clone().into_iter()
    }
}

impl<T, const N: usize> Index<usize> for Deck<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for Deck<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T, const N: usize> From<[T; N]> for Deck<T, N> {
    fn from(value: [T; N]) -> Self {
        Self(value)
    }
}

impl<T: Copy, const N: usize> From<&[T]> for Deck<T, N> {
    fn from(value: &[T]) -> Self {
        if value.len() != N {
            panic!("Incorrect length {}", value.len());
        }
        let mut result = [value[0]; N];
        for (i, e) in value.into_iter().enumerate() {
            result[i] = *e;
        }
        Self(result)
    }
}

impl<const N: usize> TurnProfile<N> {
    pub fn turn_iter(&self) -> impl Iterator<Item = &Turn> {
        self.0.iter()
    }
}

impl<const M: usize> EnergyProfile<M> {
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = (Energy, &mut u8)> {
        self.0
            .iter_mut()
            .enumerate()
            .map(|(e, n)| (Energy::from(e as u8), n))
    }
}

impl<const M: usize> Index<Energy> for EnergyProfile<M> {
    type Output = u8;

    fn index(&self, index: Energy) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

impl<const M: usize> IndexMut<Energy> for EnergyProfile<M> {
    fn index_mut(&mut self, index: Energy) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}

impl<const M: usize> Default for EnergyProfile<M> {
    fn default() -> Self {
        Self([0; M])
    }
}

impl<const N: usize> From<CostProfile<N>> for Deck<Energy, N> {
    fn from(cp: CostProfile<N>) -> Self {
        let mut deck: [Energy; N] = [Energy::default(); N];
        let mut id = 0;
        for (cost, amount) in cp.into_iter().enumerate() {
            for _ in 0..amount {
                deck[id] = Energy::from(cost as u8);
                id += 1;
            }
        }
        Self(deck)
    }
}

impl<const N: usize> IntoIterator for CostProfile<N> {
    type Item = u8;
    type IntoIter = std::array::IntoIter<u8, { MAX_COST as usize }>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(id = {}, cost = {})", self.0 >> 4, self.0 << 4 >> 4)
    }
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(id = {}, cost = {})", self.0 >> 4, self.0 << 4 >> 4)
    }
}

impl<T: std::fmt::Debug, const N: usize> std::fmt::Display for Deck<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl Default for CardIdentity {
    fn default() -> Self {
        Self::None
    }
}

impl CardIdentity {
    pub fn id(&self) -> Option<Id> {
        if let CardIdentity::Full(card) = self {
            Some(card.id())
        } else {
            None
        }
    }
}

impl<const N: usize> From<[Turn; N]> for TurnProfile<N> {
    fn from(value: [Turn; N]) -> Self {
        Self(value)
    }
}

impl From<u8> for Energy {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Into<u64> for Energy {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

impl From<u8> for TurnNumber {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<u8> for Id {
    fn from(val: u8) -> Self {
        Self(val)
    }
}

#[cfg(test)]
mod tests {
    use crate::deck::{Card, Id};

    #[test]
    fn card_id() {
        let card = Card(0b00010001);
        assert_eq!(card.id(), Id(0b00000001));
    }
}
