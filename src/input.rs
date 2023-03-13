use crate::analyse::{standard_turn_profile, AnalysisExecutor};
use crate::condition::{AllOf, AnyOf, LockConditionResult};
use crate::cost_efficiency::CostEfficiencyAnalysis;
use crate::deck::{Card, CardIdentity, Deck, Energy, TurnNumber};
use crate::MAX_COST;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct Input {
    pub(crate) cost_profile: Vec<u8>,
    pub(crate) cards: Vec<CardName>,
    pub(crate) condition_references: Vec<ConditionWithName>,
    pub(crate) analysis: Vec<Analysis>,
}

#[derive(Deserialize, Clone)]
pub struct CardName {
    pub(crate) name: String,
    pub(crate) cost: u8,
}

#[derive(Deserialize, Clone)]
pub struct Analysis {
    pub(crate) kind: String,
    pub(crate) name: String,
    pub(crate) conditions: Option<Vec<Condition>>,
}

#[derive(Deserialize, Clone)]
pub struct ConditionWithName {
    pub(crate) name: String,
    pub(crate) condition: Condition,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Condition {
    CardCondition(CardCondition),
    AllOfCondition(AllOfCondition),
    AnyOfCondition(AnyOfCondition),
    ReferenceCondition(ReferenceCondition),
}

#[derive(Deserialize, Clone)]
pub struct CardCondition {
    pub(crate) card_name: String,
    pub(crate) comes_at_or_before: u8,
}

#[derive(Deserialize, Clone)]
pub struct AllOfCondition {
    pub(crate) all_of: Vec<Condition>,
}

#[derive(Deserialize, Clone)]
pub struct AnyOfCondition {
    pub(crate) any_of: Vec<Condition>,
}

#[derive(Deserialize, Clone)]
pub struct ReferenceCondition {
    pub(crate) reference: String,
}

pub enum Error {
    Kind,
    ProfileLength(usize, usize),
    ProfileCardCount(usize, usize),
    CardNameDuplicate(String),
    NoFreeCardWithCost(String, usize),
    UnknownCardNameAnalysis(String),
    UnknownConditionReference(String),
    CardCost(usize),
    SameReference(String),
}

pub fn read_from_file(path: &str) -> Input {
    let file = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&file).unwrap()
}

pub fn parse<const N: usize>(input: Input) -> Result<Vec<AnalysisExecutor<N>>, Error> {
    let max_card_count = N;
    let max_card_cost = MAX_COST as usize;
    let Input {
        cost_profile,
        cards,
        condition_references,
        analysis,
    } = input;
    if cost_profile.len() != max_card_cost + 1 {
        return Err(Error::ProfileLength(cost_profile.len(), max_card_cost + 1));
    }
    let card_count = cost_profile.iter().map(|c| usize::from(*c)).sum();
    if card_count != max_card_count {
        return Err(Error::ProfileCardCount(card_count, max_card_count));
    }
    let mut parsed_cost_profile: [u8; { MAX_COST + 1 } as usize] = [0; { MAX_COST + 1 } as usize];
    for (i, amount) in cost_profile.into_iter().enumerate() {
        parsed_cost_profile[i] = amount;
    }

    let name_to_id = associate_card_name_with_card_id(cards, parsed_cost_profile)?;
    let named_conditions = extract_condition_references(condition_references)?;
    let result = analysis
        .into_iter()
        .map(|a| map_analysis::<N>(a, &name_to_id, &named_conditions, parsed_cost_profile))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(result)
}

fn map_analysis<const N: usize>(
    analysis: Analysis,
    name_to_id: &HashMap<String, Card>,
    named_conditions: &HashMap<String, Condition>,
    cost_profile: [u8; { MAX_COST + 1 } as usize],
) -> Result<AnalysisExecutor<N>, Error> {
    let Analysis {
        kind,
        name,
        conditions,
    } = analysis;
    match (kind.as_ref(), conditions) {
        ("custom", Some(conditions)) => {
            map_custom_analysis(name, conditions, name_to_id, named_conditions)
        }
        ("cost_efficiency", None) => map_cost_efficiency(name, cost_profile),
        _ => Err(Error::Kind),
    }
}

fn map_custom_analysis<const N: usize>(
    name: String,
    conditions: Vec<Condition>,
    name_to_id: &HashMap<String, Card>,
    named_conditions: &HashMap<String, Condition>,
) -> Result<AnalysisExecutor<N>, Error> {
    let mut deck = Vec::new();
    let mut analysis_conditions: Vec<Box<dyn crate::condition::Condition>> = Vec::new();
    for condition in conditions {
        enrich_condition(
            name_to_id,
            named_conditions,
            &mut analysis_conditions,
            &mut deck,
            condition,
        )?;
        // match condition {
        //     Condition::CardCondition(card_condition) => {}
        //     Condition::AllOfCondition(_) => {}
        //     Condition::AnyOfCondition(_) => {}
        //     Condition::ReferenceCondition(_) => {}
        // }
        // let Condition {
        //     card_name,
        //     comes_at_or_before,
        // } = condition;
        // let card_id_and_cost = name_to_id
        //     .get(&card_name)
        //     .ok_or(Error::UnknownCardNameAnalysis(card_name.clone()))?;
        // let card_identity = CardIdentity::Full(*card_id_and_cost);
        // if !deck.contains(&card_identity) {
        //     deck.push(card_identity);
        // }
        // let c = crate::condition::AllOf::new(vec![
        //     Box::new(crate::condition::CardIdCondition::new(
        //         card_id_and_cost.id(),
        //     )),
        //     Box::new(crate::condition::ComesAtOrBeforeCondition::new(
        //         TurnNumber::from(comes_at_or_before),
        //     )),
        // ]);
        // analysis_conditions.push(Box::new(crate::condition::LockConditionResult::new(c)));
    }

    for _ in 0..N - deck.len() {
        deck.push(CardIdentity::None);
    }

    let analysis = crate::condition_count::ConditionCount::new(
        name,
        crate::condition::AllOf::new(analysis_conditions),
    );

    Ok(AnalysisExecutor::new(
        Deck::<CardIdentity, N>::from(&deck[..]),
        standard_turn_profile(),
        vec![Box::new(analysis)],
    ))
}

fn enrich_condition(
    name_to_id: &HashMap<String, Card>,
    named_conditions: &HashMap<String, Condition>,
    conditions: &mut Vec<Box<dyn crate::condition::Condition>>,
    deck: &mut Vec<CardIdentity>,
    condition: Condition,
) -> Result<(), Error> {
    match condition {
        Condition::CardCondition(card_condition) => {
            let CardCondition {
                card_name,
                comes_at_or_before,
            } = card_condition;
            let card_id_and_cost = name_to_id
                .get(&card_name)
                .ok_or(Error::UnknownCardNameAnalysis(card_name.clone()))?;
            let card_identity = CardIdentity::Full(*card_id_and_cost);
            if !deck.contains(&card_identity) {
                deck.push(card_identity);
            }
            let c = crate::condition::AllOf::new(vec![
                Box::new(crate::condition::CardIdCondition::new(
                    card_id_and_cost.id(),
                )),
                Box::new(crate::condition::ComesAtOrBeforeCondition::new(
                    TurnNumber::from(comes_at_or_before),
                )),
            ]);
            conditions.push(Box::new(crate::condition::LockConditionResult::new(c)));
        }
        Condition::AllOfCondition(all_of_condition) => {
            let AllOfCondition { all_of } = all_of_condition;
            let mut child_conditions: Vec<Box<dyn crate::condition::Condition>> = Vec::new();
            for child_condition in all_of {
                enrich_condition(
                    name_to_id,
                    named_conditions,
                    &mut child_conditions,
                    deck,
                    child_condition,
                )?;
            }
            conditions.push(Box::new(LockConditionResult::new(AllOf::new(
                child_conditions,
            ))));
        }
        Condition::AnyOfCondition(any_of_condition) => {
            let AnyOfCondition { any_of } = any_of_condition;
            let mut child_conditions: Vec<Box<dyn crate::condition::Condition>> = Vec::new();
            for child_condition in any_of {
                enrich_condition(
                    name_to_id,
                    named_conditions,
                    &mut child_conditions,
                    deck,
                    child_condition,
                )?;
            }
            conditions.push(Box::new(LockConditionResult::new(AnyOf::new(
                child_conditions,
            ))));
        }
        Condition::ReferenceCondition(reference_condition) => {
            let ReferenceCondition { reference } = reference_condition;
            let referenced_condition = named_conditions
                .get(&reference)
                .cloned()
                .ok_or(Error::UnknownConditionReference(reference))?;
            enrich_condition(
                name_to_id,
                named_conditions,
                conditions,
                deck,
                referenced_condition,
            )?;
        }
    }
    Ok(())
}

fn extract_condition_references(
    condition_references: Vec<ConditionWithName>,
) -> Result<HashMap<String, Condition>, Error> {
    let mut references = HashMap::new();
    for reference in condition_references {
        if references.contains_key(&reference.name) {
            return Err(Error::SameReference(reference.name));
        }
        references.insert(reference.name, reference.condition);
    }
    Ok(references)
}

fn map_cost_efficiency<const N: usize>(
    name: String,
    cost_profile: [u8; { MAX_COST + 1 } as usize],
) -> Result<AnalysisExecutor<N>, Error> {
    let mut deck = Vec::new();

    for (cost, amount) in cost_profile.into_iter().enumerate() {
        for _ in 0..amount {
            deck.push(CardIdentity::Cost(Energy::from(cost as u8)));
        }
    }

    let analysis = CostEfficiencyAnalysis::<{ (MAX_COST + 1) as usize }>::new(name);

    Ok(AnalysisExecutor::new(
        Deck::<CardIdentity, N>::from(&deck[..]),
        standard_turn_profile(),
        vec![Box::new(analysis)],
    ))
}

fn associate_card_name_with_card_id(
    cards: Vec<CardName>,
    mut cost_profile: [u8; { MAX_COST + 1 } as usize],
) -> Result<HashMap<String, Card>, Error> {
    let mut name_to_id = HashMap::new();

    let mut next_id: u8 = 0;
    for card in cards {
        if name_to_id.contains_key(&card.name) {
            return Err(Error::CardNameDuplicate(card.name));
        }
        if card.cost > MAX_COST {
            return Err(Error::CardCost(card.cost as usize));
        }
        if cost_profile[card.cost as usize] == 0 {
            return Err(Error::NoFreeCardWithCost(
                card.name.clone(),
                card.cost as usize,
            ));
        }
        cost_profile[card.cost as usize] -= 1;
        name_to_id.insert(card.name.clone(), Card::new(next_id, card.cost));
        next_id += 1;
    }

    Ok(name_to_id)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ProfileLength(actual, correct) => write!(
                f,
                "Cost profile length ({}) should be equal to {}",
                actual, correct
            ),
            Error::ProfileCardCount(actual, correct) => write!(
                f,
                "Sum of numbers in cost profile ({}) should be equal to {}",
                actual, correct
            ),
            Error::CardNameDuplicate(name) => write!(
                f,
                "Card with name \"{}\" is listed more that once in card list",
                name
            ),
            Error::NoFreeCardWithCost(name, cost) => write!(
                f,
                "No room for {} cost card \"{}\" in cost profile left",
                cost, name
            ),
            Error::UnknownCardNameAnalysis(name) => {
                write!(f, "Card name \"{}\" in analysis is unknown", name)
            }
            Error::CardCost(cost) => write!(f, "Invalid card cost {}", cost),
            Error::Kind => write!(f, "Invalid analysis kind or format"),
            Error::SameReference(reference_name) => write!(
                f,
                "Condition reference \"{}\" exists at least twice",
                reference_name
            ),
            Error::UnknownConditionReference(reference_name) => write!(
                f,
                "Condition reference \"{}\" in analysis is unknown",
                reference_name
            ),
        }
    }
}
