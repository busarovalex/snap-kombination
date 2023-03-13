use crate::analyse::*;
use crate::condition::*;
use crate::condition_count::ConditionCount;
use crate::cost_efficiency::CostEfficiencyAnalysis;
use crate::deck::*;
use crate::permutation_optimized::DeckPermutationIterator;
use std::collections::HashMap;

#[test]
fn three_card_deck_first_card() {
    test_three_card_deck_comes_at_or_before_condition(0, "3", "1");
}

#[test]
fn three_card_deck_second_or_earlier_card() {
    test_three_card_deck_comes_at_or_before_condition(1, "3", "2");
}

#[test]
fn three_card_deck_third_or_earlier_card() {
    test_three_card_deck_comes_at_or_before_condition(2, "3", "3");
}

fn test_three_card_deck_comes_at_or_before_condition(
    comes_at_or_before: u8,
    total_amount: &str,
    count: &str,
) {
    let deck: Deck<CardIdentity, 3> =
        Deck::from([card(0, 0), CardIdentity::None, CardIdentity::None]);
    let condition = AllOf::new(vec![
        Box::new(CardIdCondition::new(Id::from(0))),
        Box::new(ComesAtOrBeforeCondition::new(TurnNumber::from(
            comes_at_or_before,
        ))),
    ]);
    let analysis = ConditionCount::new("should be in n of cases".to_string(), condition);
    let turn_profile = TurnProfile::from([turn(0, 0), turn(1, 0), turn(2, 0)]);
    let analyse = AnalysisExecutor::<3>::new(deck, turn_profile, vec![Box::new(analysis)]);
    let result = analyse
        .execute::<DeckPermutationIterator<_, 3>>(SuppressWarnings::Yes)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let result_map = result.as_map();
    assert_eq!(
        result_map.get("total_amount").map(String::as_str),
        Some(total_amount)
    );
    assert_eq!(result_map.get("count").map(String::as_str), Some(count));
}

#[test]
fn test_four_card_deck() {
    let condition = LockConditionResult::new(AllOf::new(vec![
        comes_at_or_before_condition(Id::from(0), 1),
        comes_at_or_before_condition(Id::from(1), 2),
    ]));
    let analysis = ConditionCount::new("should be in 4 of 12 cases".to_string(), condition);
    let deck: Deck<CardIdentity, 4> = Deck::from([
        card(0, 0),
        card(1, 0),
        CardIdentity::None,
        CardIdentity::None,
    ]);
    let turn_profile = TurnProfile::from([turn(0, 0), turn(1, 0), turn(2, 0), turn(3, 0)]);
    let analyse = AnalysisExecutor::<4>::new(deck, turn_profile, vec![Box::new(analysis)]);
    let result = analyse
        .execute::<DeckPermutationIterator<_, 4>>(SuppressWarnings::Yes)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let result_map = result.as_map();
    assert_eq!(
        result_map.get("total_amount").map(String::as_str),
        Some("12")
    );
    assert_eq!(result_map.get("count").map(String::as_str), Some("4"));
}

fn comes_at_or_before_condition(id: Id, comes_at_or_before: u8) -> Box<dyn Condition> {
    Box::new(LockConditionResult::new(AllOf::new(vec![
        Box::new(CardIdCondition::new(id)),
        Box::new(ComesAtOrBeforeCondition::new(TurnNumber::from(
            comes_at_or_before,
        ))),
    ])))
}

fn card(id: u8, cost: u8) -> CardIdentity {
    CardIdentity::Full(Card::new(id, cost))
}

#[test]
fn cost_efficiency_with_all_one_cost_cards() {
    let deck: Deck<CardIdentity, 4> = Deck::from([CardIdentity::Cost(Energy::from(1)); 4]);

    let result_map = calculate_cost_efficiency_for_4_card_deck(deck);
    assert_eq!(result_map.get("total_spent").map(String::as_str), Some("4"));
    assert_eq!(
        result_map.get("number_of_decks").map(String::as_str),
        Some("1")
    );
}

#[test]
fn cost_efficiency_with_all_four_cost_cards() {
    let deck: Deck<CardIdentity, 4> = Deck::from([CardIdentity::Cost(Energy::from(4)); 4]);

    let result_map = calculate_cost_efficiency_for_4_card_deck(deck);
    assert_eq!(result_map.get("total_spent").map(String::as_str), Some("4"));
    assert_eq!(
        result_map.get("number_of_decks").map(String::as_str),
        Some("1")
    );
}

fn calculate_cost_efficiency_for_4_card_deck(
    deck: Deck<CardIdentity, 4>,
) -> HashMap<String, String> {
    let analysis = CostEfficiencyAnalysis::<7>::new("Cost efficiiency".to_string());

    let turn_profile = TurnProfile::from([turn(1, 1), turn(2, 2), turn(3, 3), turn(4, 4)]);
    let analyse = AnalysisExecutor::<4>::new(deck, turn_profile, vec![Box::new(analysis)]);
    let result = analyse
        .execute::<DeckPermutationIterator<_, 4>>(SuppressWarnings::Yes)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    result.as_map()
}

fn turn(number: u8, energy: u8) -> Turn {
    Turn {
        number: TurnNumber::from(number),
        energy: Energy::from(energy),
    }
}
