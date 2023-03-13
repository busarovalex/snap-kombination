use crate::analyse::{Analysis, AnalysisResult};
use crate::condition::{Condition, LockConditionResult};
use crate::deck::{CardIdentity, Turn};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ConditionCount<T> {
    name: String,
    condition: LockConditionResult<T>,
    count: u64,
    total_amount: u64,
}

#[derive(Debug)]
pub struct ConditionCountResult {
    name: String,
    count: u64,
    total_amount: u64,
}

impl<T> ConditionCount<T> {
    pub fn new(name: String, condition: T) -> Self {
        ConditionCount {
            name,
            condition: LockConditionResult::new(condition),
            count: 0,
            total_amount: 0,
        }
    }
}

impl<T: Condition> Analysis for ConditionCount<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn accept(&mut self, card: CardIdentity, turn: Turn) {
        self.condition.accept(card, turn);
    }

    fn next_deck(&mut self) {
        self.total_amount += 1;
        if self.condition.result() {
            self.count += 1;
        }
        self.condition.next_deck();
    }

    fn result(&self) -> Box<dyn AnalysisResult> {
        Box::new(ConditionCountResult {
            name: self.name.clone(),
            count: self.count,
            total_amount: self.total_amount,
        })
    }
}

impl AnalysisResult for ConditionCountResult {
    fn as_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("name".to_owned(), self.name.clone());
        map.insert("count".to_owned(), format!("{}", self.count));
        map.insert("total_amount".to_owned(), format!("{}", self.total_amount));
        map
    }
}

impl std::fmt::Display for ConditionCountResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let success_percent = (self.count as f32) / (self.total_amount as f32) * 100.0;
        write!(
            f,
            "{} is available {:.2} percent of the time",
            self.name, success_percent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{AllOf, CardIdCondition, ComesAtOrBeforeCondition};
    use crate::deck::{Card, Energy, Id, TurnNumber};

    #[test]
    fn test_condition_count() {
        let mut analysis = analysis_comes_at_or_before(0);

        analysis.accept(card(0, 0), turn(0, 0));
        analysis.next_deck();

        let result_map = analysis.result().as_map();
        assert_eq!(result_map.get("count").map(String::as_str), Some("1"));
    }

    #[test]
    fn test_condition_count_2() {
        let mut analysis = analysis_comes_at_or_before(0);

        analysis.accept(card(0, 0), turn(1, 0));
        analysis.next_deck();

        let result_map = analysis.result().as_map();
        assert_eq!(result_map.get("count").map(String::as_str), Some("0"));
    }

    #[test]
    fn test_condition_count_3() {
        let mut analysis = analysis_comes_at_or_before(0);

        analysis.accept(card(1, 0), turn(0, 0));
        analysis.accept(card(0, 0), turn(1, 0));
        analysis.next_deck();

        let result_map = analysis.result().as_map();
        assert_eq!(result_map.get("count").map(String::as_str), Some("0"));
    }

    #[test]
    fn test_condition_count_4() {
        let mut analysis = analysis_comes_at_or_before(1);

        analysis.accept(card(1, 0), turn(0, 0));
        analysis.accept(card(0, 0), turn(1, 0));
        analysis.next_deck();

        let result_map = analysis.result().as_map();
        assert_eq!(result_map.get("count").map(String::as_str), Some("1"));
    }

    #[test]
    fn test_condition_count_5() {
        let mut analysis = analysis_comes_at_or_before(1);

        analysis.accept(card(1, 0), turn(0, 0));
        analysis.accept(card(0, 0), turn(1, 0));
        analysis.accept(card(2, 0), turn(2, 0));
        analysis.next_deck();

        let result_map = analysis.result().as_map();
        assert_eq!(result_map.get("count").map(String::as_str), Some("1"));
    }

    fn analysis_comes_at_or_before(position: u8) -> ConditionCount<AllOf> {
        let condition = analysis_comes_at_or_before_with_id(position, 0);
        ConditionCount::new("test".to_string(), condition)
    }

    #[test]
    fn test_condition_count_6() {
        let mut analysis = analysis_2_cards_come_at_or_before(0, 1);

        analysis.accept(card(0, 0), turn(0, 0));
        analysis.accept(card(1, 0), turn(1, 0));
        analysis.next_deck();

        let result_map = analysis.result().as_map();
        assert_eq!(result_map.get("count").map(String::as_str), Some("1"));
    }

    #[test]
    fn test_condition_count_7() {
        let mut analysis = analysis_2_cards_come_at_or_before(1, 1);

        analysis.accept(card(0, 0), turn(0, 0));
        analysis.accept(card(1, 0), turn(1, 0));
        analysis.next_deck();

        let result_map = analysis.result().as_map();
        assert_eq!(result_map.get("count").map(String::as_str), Some("1"));
    }

    #[test]
    fn test_condition_count_8() {
        let mut analysis = analysis_2_cards_come_at_or_before(0, 1);

        analysis.accept(card(1, 0), turn(0, 0));
        analysis.accept(card(0, 0), turn(1, 0));
        analysis.next_deck();

        let result_map = analysis.result().as_map();
        assert_eq!(result_map.get("count").map(String::as_str), Some("0"));
    }

    #[test]
    fn test_condition_count_9() {
        let mut analysis = analysis_2_cards_come_at_or_before(0, 1);

        analysis.accept(card(0, 0), turn(0, 0));
        analysis.accept(card(1, 0), turn(1, 0));
        analysis.next_deck();

        analysis.accept(card(1, 0), turn(0, 0));
        analysis.accept(card(0, 0), turn(1, 0));
        analysis.next_deck();

        let result_map = analysis.result().as_map();
        assert_eq!(result_map.get("count").map(String::as_str), Some("1"));
    }

    fn analysis_2_cards_come_at_or_before(position_a: u8, position_b: u8) -> ConditionCount<AllOf> {
        let condition_a = analysis_comes_at_or_before_with_id(position_a, 0);
        let condition_b = analysis_comes_at_or_before_with_id(position_b, 1);

        ConditionCount::new(
            "test".to_string(),
            AllOf::new(vec![
                Box::new(LockConditionResult::new(condition_a)),
                Box::new(LockConditionResult::new(condition_b)),
            ]),
        )
    }

    fn analysis_comes_at_or_before_with_id(position: u8, id: u8) -> AllOf {
        AllOf::new(vec![
            Box::new(CardIdCondition::new(Id::from(id))),
            Box::new(ComesAtOrBeforeCondition::new(TurnNumber::from(position))),
        ])
    }

    fn card(id: u8, cost: u8) -> CardIdentity {
        CardIdentity::Full(Card::new(id, cost))
    }

    fn turn(number: u8, energy: u8) -> Turn {
        Turn {
            number: TurnNumber::from(number),
            energy: Energy::from(energy),
        }
    }
}
