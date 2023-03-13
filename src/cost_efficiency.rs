use crate::analyse::{Analysis, AnalysisResult};
use crate::deck::{CardIdentity, Energy, EnergyProfile, Turn, TurnNumber};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CostEfficiencyAnalysis<const M: usize> {
    name: String,
    total_spent: u64,
    number_of_decks: u64,
    energy_left: Energy,
    energy_spent: Energy,
    last_turn: TurnNumber,
    energy_profile: EnergyProfile<M>,
}

#[derive(Debug)]
struct CostEfficiencyAnalysisResult {
    name: String,
    total_spent: u64,
    number_of_decks: u64,
}

impl<const M: usize> CostEfficiencyAnalysis<M> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            total_spent: 0,
            number_of_decks: 0,
            energy_left: Default::default(),
            energy_spent: Default::default(),
            last_turn: Self::initial_turn_not_equal_to_others(),
            energy_profile: Default::default(),
        }
    }

    fn initial_turn_not_equal_to_others() -> TurnNumber {
        TurnNumber::from(u8::MAX)
    }

    fn accept(&mut self, cost: Energy, turn: Turn) {
        self.energy_profile[cost] += 1;
        if self.last_turn != turn.number {
            self.last_turn = turn.number;
            self.energy_left = turn.energy;
        }
        let (min_to_max_spent, min_to_max_profile) = self.spend_from_min_to_max();
        let (max_to_min_spent, max_to_min_profile) = self.spend_from_max_to_min();
        if min_to_max_spent > max_to_min_spent {
            self.energy_spent += min_to_max_spent;
            self.energy_profile = min_to_max_profile;
        } else {
            self.energy_spent += max_to_min_spent;
            self.energy_profile = max_to_min_profile;
        }
    }

    fn spend_from_min_to_max(&self) -> (Energy, EnergyProfile<M>) {
        let mut profile = self.energy_profile.clone();
        let mut left = self.energy_left.clone();
        let mut spent = Energy::default();
        for (energy, amount) in profile.iter_mut() {
            if left < energy {
                break;
            }
            if *amount > 0 {
                *amount -= 1;
                left -= energy;
                spent += energy;
            }
        }
        (spent, profile)
    }

    fn spend_from_max_to_min(&self) -> (Energy, EnergyProfile<M>) {
        let mut profile = self.energy_profile.clone();
        let mut left = self.energy_left.clone();
        let mut spent = Energy::default();
        for (energy, amount) in profile.iter_mut().rev() {
            if left < energy {
                continue;
            }
            if *amount > 0 {
                *amount -= 1;
                left -= energy;
                spent += energy;
            }
        }
        (spent, profile)
    }

    fn next_deck(&mut self) {
        let spent: u64 = self.energy_spent.into();
        self.total_spent += spent;
        self.number_of_decks += 1;
        self.last_turn = Self::initial_turn_not_equal_to_others();
        self.energy_spent = Energy::default();
        self.energy_left = Energy::default();
        self.energy_profile = EnergyProfile::default();
    }
}

impl<const M: usize> Analysis for CostEfficiencyAnalysis<M> {
    fn name(&self) -> &str {
        &self.name
    }

    fn accept(&mut self, card: CardIdentity, turn: Turn) {
        if let CardIdentity::Cost(cost) = card {
            self.accept(cost, turn);
        } else {
            panic!(
                "{} only accepts cost card identities",
                stringify!(CostEfficiencyAnalysis)
            );
        }
    }

    fn next_deck(&mut self) {
        self.next_deck();
    }

    fn result(&self) -> Box<dyn AnalysisResult> {
        Box::new(CostEfficiencyAnalysisResult {
            name: self.name.clone(),
            total_spent: self.total_spent,
            number_of_decks: self.number_of_decks,
        })
    }
}

impl AnalysisResult for CostEfficiencyAnalysisResult {
    fn as_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("name".to_owned(), self.name.clone());
        map.insert("total_spent".to_owned(), format!("{}", self.total_spent));
        map.insert(
            "number_of_decks".to_owned(),
            format!("{}", self.number_of_decks),
        );
        map
    }
}

impl std::fmt::Display for CostEfficiencyAnalysisResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: median spent: {:.1} ({} decks analysed)",
            self.name,
            self.total_spent as f64 / self.number_of_decks as f64,
            self.number_of_decks
        )
    }
}

#[cfg(test)]
mod tests {
    use super::CostEfficiencyAnalysis;
    use crate::deck::{Energy, EnergyProfile, Turn, TurnNumber};

    #[test]
    fn calculates_energy_spent() {
        let mut analysis = CostEfficiencyAnalysis::<6>::new("cost efficiency".to_owned());
        analysis.accept(Energy::from(1), turn(1, 1));
        analysis.next_deck();

        assert_eq!(analysis.total_spent, 1);
    }

    #[test]
    fn calculates_energy_spent_several_turns() {
        let mut analysis = CostEfficiencyAnalysis::<6>::new("cost efficiency".to_owned());
        analysis.accept(Energy::from(2), turn(1, 1));
        analysis.accept(Energy::from(3), turn(1, 1));
        analysis.accept(Energy::from(1), turn(1, 1));
        analysis.accept(Energy::from(3), turn(2, 2));
        analysis.next_deck();

        assert_eq!(analysis.total_spent, 3);
    }

    #[test]
    fn calculates_energy_spent_several_decks() {
        let mut analysis = CostEfficiencyAnalysis::<6>::new("cost efficiency".to_owned());
        analysis.accept(Energy::from(3), turn(1, 1));
        analysis.accept(Energy::from(2), turn(2, 2));
        analysis.accept(Energy::from(1), turn(3, 3));
        analysis.next_deck();
        assert_eq!(analysis.total_spent, 5);

        analysis.accept(Energy::from(3), turn(1, 1));
        analysis.accept(Energy::from(3), turn(2, 2));
        analysis.accept(Energy::from(3), turn(3, 3));
        analysis.next_deck();

        assert_eq!(analysis.total_spent, 8);
    }

    fn turn(number: u8, energy: u8) -> Turn {
        Turn {
            number: TurnNumber::from(number),
            energy: Energy::from(energy),
        }
    }

    #[test]
    fn spend_from_min_to_max_returns_correct_value() {
        let analysis = CostEfficiencyAnalysis {
            name: "test".to_string(),
            total_spent: 0,
            number_of_decks: 0,
            energy_left: Energy::from(3),
            energy_spent: Default::default(),
            last_turn: Default::default(),
            energy_profile: energy_profile([0, 1, 0, 1]),
        };

        let (energy, profile) = analysis.spend_from_min_to_max();

        assert_eq!(energy, Energy::from(1));
        assert_eq!(profile, energy_profile([0, 0, 0, 1]));
    }

    #[test]
    fn spend_from_max_to_min_returns_correct_value() {
        let analysis = CostEfficiencyAnalysis {
            name: "test".to_string(),
            total_spent: 0,
            number_of_decks: 0,
            energy_left: Energy::from(3),
            energy_spent: Default::default(),
            last_turn: Default::default(),
            energy_profile: energy_profile([0, 1, 0, 1]),
        };

        let (energy, profile) = analysis.spend_from_max_to_min();

        assert_eq!(energy, Energy::from(3));
        assert_eq!(profile, energy_profile([0, 1, 0, 0]));
    }

    #[test]
    fn spend_from_max_to_min_returns_correct_value_exact_equality() {
        let analysis = CostEfficiencyAnalysis {
            name: "test".to_string(),
            total_spent: 0,
            number_of_decks: 0,
            energy_left: Energy::from(2),
            energy_spent: Default::default(),
            last_turn: Default::default(),
            energy_profile: energy_profile([0, 0, 1, 0]),
        };

        let (energy, profile) = analysis.spend_from_max_to_min();

        assert_eq!(energy, Energy::from(2));
        assert_eq!(profile, energy_profile([0, 0, 0, 0]));
    }

    fn energy_profile<const N: usize>(profile: [u8; N]) -> EnergyProfile<N> {
        let mut energy_profile = EnergyProfile::default();
        for (i, val) in profile.into_iter().enumerate() {
            energy_profile[Energy::from(i as u8)] = val;
        }
        energy_profile
    }
}
