use std::collections::HashMap;
use std::convert::TryFrom;

use crate::deck::{CardIdentity, Deck, Energy, Turn, TurnNumber, TurnProfile};
use crate::PERMUTATION_COUNT_WARNING_THRESHOLD;

pub trait AnalysisResult: std::fmt::Debug + std::fmt::Display {
    fn as_map(&self) -> HashMap<String, String>;
}

pub trait Analysis: std::fmt::Debug + 'static {
    fn name(&self) -> &str;
    fn accept(&mut self, card: CardIdentity, turn: Turn);
    fn next_deck(&mut self);
    fn result(&self) -> Box<dyn AnalysisResult>;
}

pub trait PermutationIterator<T> {
    fn new(collection: T) -> Self;
    fn next(&mut self) -> Option<T>;
    fn count(&self) -> u64;
}

#[derive(Debug)]
pub struct AnalysisExecutor<const N: usize> {
    deck: crate::deck::Deck<CardIdentity, N>,
    analysis: Vec<Box<dyn Analysis>>,
    turn_profile: TurnProfile<N>,
}

#[derive(Debug)]
pub enum Warning {
    TooManyPermutations(u64),
}

#[derive(Debug, Eq, PartialEq)]
pub enum SuppressWarnings {
    Yes,
    No,
}

impl<const N: usize> AnalysisExecutor<N> {
    pub(crate) fn new(
        deck: crate::deck::Deck<CardIdentity, N>,
        turn_profile: TurnProfile<N>,
        analysis: Vec<Box<dyn Analysis>>,
    ) -> Self {
        Self {
            deck,
            analysis,
            turn_profile,
        }
    }

    pub fn execute<T>(
        mut self,
        suppress_warnings: SuppressWarnings,
    ) -> Result<Vec<Box<dyn AnalysisResult>>, (Self, Warning)>
    where
        T: PermutationIterator<Deck<CardIdentity, N>>,
    {
        let mut permutations = T::new(self.deck);
        if suppress_warnings == SuppressWarnings::No
            && permutations.count() > PERMUTATION_COUNT_WARNING_THRESHOLD
        {
            return Err((self, Warning::TooManyPermutations(permutations.count())));
        }
        while let Some(deck) = permutations.next() {
            for (card, turn) in deck.card_iter().zip(self.turn_profile.turn_iter()) {
                for analysis in self.analysis.iter_mut() {
                    analysis.accept(card, *turn);
                }
            }
            for analysis in self.analysis.iter_mut() {
                analysis.next_deck();
            }
        }
        Ok(self.analysis.iter().map(|a| a.result()).collect())
    }
}

pub fn standard_turn_profile<const N: usize>() -> TurnProfile<N> {
    let mut standard = vec![
        Turn {
            number: TurnNumber::from(1),
            energy: Energy::from(1),
        },
        Turn {
            number: TurnNumber::from(1),
            energy: Energy::from(1),
        },
        Turn {
            number: TurnNumber::from(1),
            energy: Energy::from(1),
        },
        Turn {
            number: TurnNumber::from(1),
            energy: Energy::from(1),
        },
        Turn {
            number: TurnNumber::from(2),
            energy: Energy::from(2),
        },
        Turn {
            number: TurnNumber::from(3),
            energy: Energy::from(3),
        },
        Turn {
            number: TurnNumber::from(4),
            energy: Energy::from(4),
        },
        Turn {
            number: TurnNumber::from(5),
            energy: Energy::from(5),
        },
        Turn {
            number: TurnNumber::from(6),
            energy: Energy::from(6),
        },
        Turn {
            number: TurnNumber::from(7),
            energy: Energy::from(0),
        },
        Turn {
            number: TurnNumber::from(8),
            energy: Energy::from(0),
        },
        Turn {
            number: TurnNumber::from(9),
            energy: Energy::from(0),
        },
    ];

    if standard.len() < N {
        let turn_after_12 = Turn {
            number: TurnNumber::from(9),
            energy: Energy::from(0),
        };
        standard.extend([turn_after_12].into_iter().cycle().take(N - standard.len()));
    }

    TurnProfile::from(<[Turn; N]>::try_from(&standard[0..N]).unwrap())
}

impl std::fmt::Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Warning::TooManyPermutations(count) => write!(f, "Too many permutations: {}", count),
        }
    }
}
