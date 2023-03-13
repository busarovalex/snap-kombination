use crate::analyse::PermutationIterator;
use crate::deck::Deck;

pub struct AllPermutationsIterator<T> {
    collection: Vec<T>,
    initial_returned: bool,
    i: usize,
    c: Vec<usize>,
}

impl<T: Copy, const N: usize> PermutationIterator<Deck<T, N>> for AllPermutationsIterator<T> {
    fn new(deck: Deck<T, N>) -> Self {
        Self::new(deck.card_iter().collect())
    }

    fn next(&mut self) -> Option<Deck<T, N>> {
        let next_permutation: Option<&[T]> = AllPermutationsIterator::next(self);
        next_permutation.map(<Deck<T, N>>::from)
    }

    fn count(&self) -> u64 {
        (1..=N).into_iter().map(|c| c as u64).product()
    }
}

impl<T> AllPermutationsIterator<T> {
    fn new(collection: Vec<T>) -> Self {
        let c = vec![0; collection.len()];
        Self {
            collection,
            initial_returned: false,
            i: 0,
            c,
        }
    }

    fn next(&mut self) -> Option<&[T]> {
        if !self.initial_returned {
            self.initial_returned = true;
            return Some(&self.collection);
        }
        self.next_permutation()
    }

    fn next_permutation(&mut self) -> Option<&[T]> {
        let Self {
            i, c, collection, ..
        } = self;
        while *i < collection.len() {
            if c[*i] < *i {
                if *i % 2 == 0 {
                    collection.swap(0, *i);
                } else {
                    collection.swap(c[*i], *i);
                }
                c[*i] += 1;
                *i = 0;
                return Some(collection);
            } else {
                c[*i] = 0;
                *i += 1;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::AllPermutationsIterator;
    use std::collections::HashSet;

    #[test]
    fn permutations_3_deck_test() {
        let deck = [1, 2, 3];
        let (all_permutations, unique) = collect_permutations(deck);
        let expected: HashSet<[i32; 3]> = vec![
            [1, 2, 3],
            [1, 3, 2],
            [2, 1, 3],
            [2, 3, 1],
            [3, 1, 2],
            [3, 2, 1],
        ]
        .into_iter()
        .collect();
        assert_eq!(all_permutations.len(), 6);
        assert_eq!(unique, expected);
    }

    #[test]
    fn permutations_1_deck_test() {
        let deck = [1];
        let (all_permutations, unique) = collect_permutations(deck);
        let expected: HashSet<[i32; 1]> = vec![[1]].into_iter().collect();
        assert_eq!(all_permutations.len(), 1);
        assert_eq!(unique, expected);
    }

    #[test]
    fn permutations_4_deck_test() {
        let deck = [1, 2, 3, 4];
        let (all_permutations, unique) = collect_permutations(deck);
        let expected: HashSet<[i32; 4]> = vec![
            [1, 2, 3, 4],
            [1, 2, 4, 3],
            [1, 3, 2, 4],
            [1, 3, 4, 2],
            [1, 4, 2, 3],
            [1, 4, 3, 2],
            [2, 1, 3, 4],
            [2, 1, 4, 3],
            [2, 3, 1, 4],
            [2, 3, 4, 1],
            [2, 4, 1, 3],
            [2, 4, 3, 1],
            [3, 1, 2, 4],
            [3, 1, 4, 2],
            [3, 2, 1, 4],
            [3, 2, 4, 1],
            [3, 4, 1, 2],
            [3, 4, 2, 1],
            [4, 1, 2, 3],
            [4, 1, 3, 2],
            [4, 2, 1, 3],
            [4, 2, 3, 1],
            [4, 3, 1, 2],
            [4, 3, 2, 1],
        ]
        .into_iter()
        .collect();
        assert_eq!(all_permutations.len(), 24);
        assert_eq!(unique, expected);
    }

    fn collect_permutations<const N: usize>(deck: [i32; N]) -> (Vec<[i32; N]>, HashSet<[i32; N]>) {
        let mut unique = HashSet::with_capacity((1..N).product());
        let mut all = Vec::with_capacity((1..N).product());
        let mut permutations = AllPermutationsIterator::new(Vec::from(deck));
        while let Some(next_permutation) = permutations.next() {
            let mut c = [0; N];
            for (i, v) in next_permutation.iter().enumerate() {
                c[i] = *v;
            }
            unique.insert(c);
            all.push(c);
        }
        (all, unique)
    }
}
