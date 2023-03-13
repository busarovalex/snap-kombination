use crate::deck::{CardIdentity, Id, Turn, TurnNumber};

pub trait Condition: std::fmt::Debug + 'static {
    fn check(&mut self, card: CardIdentity, turn: Turn) -> bool;
    fn next_deck(&mut self);
}

#[derive(Debug)]
pub struct AllOf {
    all: Vec<Box<dyn Condition>>,
}

#[derive(Debug)]
pub struct AnyOf {
    any: Vec<Box<dyn Condition>>,
}

#[derive(Debug)]
pub struct CardIdCondition(crate::deck::Id);

#[derive(Debug)]
pub struct ComesAtOrBeforeCondition(TurnNumber);

#[derive(Debug)]
pub struct LockConditionResult<T> {
    result: bool,
    condition: T,
}

impl AllOf {
    pub fn new(all: Vec<Box<dyn Condition>>) -> Self {
        AllOf { all }
    }
}

impl AnyOf {
    pub fn new(any: Vec<Box<dyn Condition>>) -> Self {
        AnyOf { any }
    }
}

impl<T> LockConditionResult<T> {
    pub fn new(condition: T) -> Self {
        LockConditionResult {
            result: false,
            condition,
        }
    }
}

impl Condition for CardIdCondition {
    fn check(&mut self, card: CardIdentity, _turn: Turn) -> bool {
        Some(self.0) == card.id()
    }

    fn next_deck(&mut self) {
        //do nothing
    }
}

impl CardIdCondition {
    pub fn new(id: Id) -> CardIdCondition {
        CardIdCondition(id)
    }
}

impl Condition for ComesAtOrBeforeCondition {
    fn check(&mut self, _card: CardIdentity, turn: Turn) -> bool {
        turn.number <= self.0
    }

    fn next_deck(&mut self) {
        //do nothing
    }
}

impl ComesAtOrBeforeCondition {
    pub fn new(turn_number: TurnNumber) -> Self {
        ComesAtOrBeforeCondition(turn_number)
    }
}

impl Condition for AllOf {
    fn check(&mut self, card: CardIdentity, turn: Turn) -> bool {
        let mut result = true;
        for condition in self.all.iter_mut() {
            result = result & condition.check(card, turn);
        }
        result
    }

    fn next_deck(&mut self) {
        self.all.iter_mut().for_each(|c| c.next_deck());
    }
}

impl Condition for AnyOf {
    fn check(&mut self, card: CardIdentity, turn: Turn) -> bool {
        let mut result = false;
        for condition in self.any.iter_mut() {
            result = result | condition.check(card, turn);
        }
        result
    }

    fn next_deck(&mut self) {
        self.any.iter_mut().for_each(|c| c.next_deck());
    }
}

impl<T: Condition> Condition for LockConditionResult<T> {
    fn check(&mut self, card: CardIdentity, turn: Turn) -> bool {
        if self.result {
            return true;
        }
        self.result = self.result | self.condition.check(card, turn);
        self.result
    }

    fn next_deck(&mut self) {
        self.result = false;
        self.condition.next_deck();
    }
}

impl<T: Condition> LockConditionResult<T> {
    pub fn accept(&mut self, card: CardIdentity, turn: Turn) {
        if self.result {
            return;
        }
        self.result = self.result | self.condition.check(card, turn);
    }

    pub fn result(&self) -> bool {
        self.result
    }
}
