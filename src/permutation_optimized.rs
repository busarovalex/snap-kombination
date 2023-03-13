use crate::analyse::PermutationIterator;
use crate::deck::Deck;
use crate::placement::{Placement, PlacementIterator};

pub struct DeckPermutationIterator<T, const N: usize> {
    placement_iterators: [PlacementIterator<N>; N],
    placements: [Placement<N>; N],
    placement_mappings: [PlacementMapping<N>; N],
    deck_parts: [(T, usize); N],
    len: usize,
    next_deck: Option<Deck<T, N>>,
    sample: Deck<T, N>,
}

impl<T: std::fmt::Debug + Default + Copy + Eq, const N: usize> PermutationIterator<Deck<T, N>>
    for DeckPermutationIterator<T, N>
{
    fn new(deck: Deck<T, N>) -> Self {
        DeckPermutationIterator::new(deck)
    }

    fn next(&mut self) -> Option<Deck<T, N>> {
        DeckPermutationIterator::next(self)
    }

    fn count(&self) -> u64 {
        let mut n = N as u64;
        let mut count: u64 = 1;
        for k in self.deck_parts[0..self.len]
            .iter()
            .map(|(_, k)| *k as u64)
            .skip(1)
        {
            count *= (n - k + 1..=n).product::<u64>() / (1..=k).product::<u64>();
            n -= k;
        }
        count
    }
}

#[derive(Debug, Copy, Clone)]
struct PlacementMapping<const N: usize>([usize; N]);

impl<T: Default + Copy + Eq + std::fmt::Debug, const N: usize> DeckPermutationIterator<T, N> {
    pub fn new(deck: Deck<T, N>) -> Self {
        let deck_parts = Self::split_by_identity(deck);
        let len = deck_parts.iter().filter(|(_, count)| *count > 0).count();

        let mut placement_iterators: [PlacementIterator<N>; N] = [PlacementIterator::default(); N];
        let mut placements: [Placement<N>; N] = [Placement::default(); N];
        let mut placement_mappings: [PlacementMapping<N>; N] = [PlacementMapping::identity(); N];

        let mut total_places = N;
        for (i, (_card, count)) in deck_parts[0..len].iter().enumerate() {
            placement_iterators[i] = PlacementIterator::new(total_places, *count);
            placements[i] = placement_iterators[i].next().unwrap();
            total_places -= *count;
            if i == 0 {
                continue;
            }
            let (mappings_before, mappings_after) = placement_mappings.split_at_mut(i);
            let current_mapping = &mut mappings_after[0];
            *current_mapping = PlacementMapping::from_placement_positions_and_mapping(
                placements[i - 1].positions(),
                mappings_before.last().unwrap().clone(),
            );
        }

        let next_deck = Some(Self::do_get_current_deck(
            deck,
            &placements[0..len],
            &placement_mappings[0..len],
            &deck_parts[0..len],
        ));

        Self {
            placement_iterators,
            placements,
            placement_mappings,
            deck_parts,
            len,
            next_deck,
            sample: deck,
        }
    }

    pub fn next(&mut self) -> Option<Deck<T, N>> {
        let deck = self.next_deck.take()?;
        let mut mapping_update_index = 0;
        let mut finished = false;
        for (index, placement_iterator) in self.placement_iterators[0..self.len]
            .iter_mut()
            .enumerate()
            .rev()
        {
            mapping_update_index = index;
            if let Some(placement) = placement_iterator.next() {
                self.placements[index] = placement;
                break;
            } else if index == 0 {
                finished = true;
            } else {
                placement_iterator.reset();
                self.placements[index] = placement_iterator.next().unwrap();
            }
        }
        if mapping_update_index != self.len - 1 {
            self.update_mappings(mapping_update_index);
        }
        if !finished {
            self.next_deck = Some(self.get_current_deck());
        }
        Some(deck)
    }

    fn update_mappings(&mut self, index: usize) {
        for i in (index..self.len).into_iter().filter(|i| *i > 0) {
            let (mappings_before, mappings_after) = self.placement_mappings.split_at_mut(i);
            let current_mapping = &mut mappings_after[0];
            *current_mapping = PlacementMapping::from_placement_positions_and_mapping(
                self.placements[i - 1].positions(),
                mappings_before.last().unwrap().clone(),
            );
        }
    }

    fn get_current_deck(&self) -> Deck<T, N> {
        Self::do_get_current_deck(
            self.sample,
            &self.placements[0..self.len],
            &self.placement_mappings[0..self.len],
            &self.deck_parts[0..self.len],
        )
    }

    fn do_get_current_deck(
        sample: Deck<T, N>,
        placements: &[Placement<N>],
        mappings: &[PlacementMapping<N>],
        deck_parts: &[(T, usize)],
    ) -> Deck<T, N> {
        let mut deck = sample;
        for ((placement, mapping), card) in placements
            .iter()
            .zip(mappings.iter())
            .zip(deck_parts.iter().map(|(card, _)| *card))
        {
            for position in placement.positions().iter().map(|i| mapping.map(*i)) {
                deck[position] = card;
            }
        }
        deck
    }

    fn split_by_identity(deck: Deck<T, N>) -> [(T, usize); N] {
        let mut deck_parts = [(T::default(), 0); N];
        deck_parts.sort_by(|(_card_a, amount_a), (_card_b, amount_b)| amount_b.cmp(amount_a));
        'outer: for card in deck.card_iter() {
            for (existing_value, count) in deck_parts.iter_mut() {
                if *existing_value == card || *count == 0 {
                    *existing_value = card;
                    *count += 1;
                    continue 'outer;
                }
            }
        }
        deck_parts
    }
}

impl<const N: usize> PlacementMapping<N> {
    fn from_placement_positions_and_mapping(placement: &[usize], mapping: Self) -> Self {
        let mut mapping = mapping.0;
        for occupied_position in placement {
            mapping[*occupied_position] = usize::MAX;
        }
        let mut mapping_without_occupied_positions = [N - 1; N];
        let mut i = 0;
        let mut j = 0;
        while i < N {
            if let Some(old_mapping_value) = mapping.get(j).cloned() {
                if old_mapping_value != usize::MAX {
                    mapping_without_occupied_positions[i] = old_mapping_value;
                    i += 1;
                    j += 1;
                } else {
                    j += 1;
                }
            } else {
                mapping_without_occupied_positions[i] = N - 1;
                i += 1;
            }
        }
        Self(mapping_without_occupied_positions)
    }

    fn identity() -> Self {
        let mut mapping = [0; N];
        for i in 0..N {
            mapping[i] = i;
        }
        Self(mapping)
    }

    fn map(&self, index: usize) -> usize {
        self.0[index]
    }
}

#[cfg(test)]
mod tests {
    use super::{DeckPermutationIterator, PlacementMapping};
    use crate::analyse::PermutationIterator;
    use crate::deck::Deck;
    use std::collections::HashSet;

    #[test]
    fn test_unique_permutations_count_2_1_1() {
        let deck: Deck<usize, 4> = new_deck([0, 0, 1, 2]);
        let result = DeckPermutationIterator::new(deck).count();

        assert_eq!(result, 12);
    }

    #[test]
    fn test_unique_permutations_count_1_11() {
        let deck: Deck<usize, 12> = new_deck([0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let result = DeckPermutationIterator::new(deck).count();

        assert_eq!(result, 12);
    }

    #[test]
    fn test_unique_permutations_count_1_1_1_9() {
        let deck: Deck<usize, 12> = new_deck([0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
        let result = DeckPermutationIterator::new(deck).count();

        assert_eq!(result, 1320);
    }

    #[test]
    fn test_unique_permutations_count_3_3_3_2_1() {
        let deck: Deck<usize, 12> = new_deck([0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4]);
        let result = DeckPermutationIterator::new(deck).count();

        assert_eq!(result, 1108800);
    }

    #[test]
    fn test_1_unique_card_size_2_deck() {
        let deck: Deck<usize, 2> = new_deck([0, 0]);
        let (count, decks) = collect_new_deck_permutations(deck);

        let expected: HashSet<Deck<usize, 2>> =
            vec![[0, 0]].into_iter().map(self::new_deck).collect();

        assert_eq!(decks, expected);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_2_unique_card_size_2_deck() {
        let deck: Deck<usize, 2> = new_deck([0, 1]);
        let (count, decks) = collect_new_deck_permutations(deck);

        let expected: HashSet<Deck<usize, 2>> = vec![[0, 1], [1, 0]]
            .into_iter()
            .map(self::new_deck)
            .collect();

        assert_eq!(decks, expected);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_2_unique_card_size_3_deck() {
        let deck: Deck<usize, 3> = new_deck([0, 1, 1]);
        let (count, decks) = collect_new_deck_permutations(deck);

        let expected: HashSet<Deck<usize, 3>> = vec![[1, 1, 0], [1, 0, 1], [0, 1, 1]]
            .into_iter()
            .map(self::new_deck)
            .collect();

        assert_eq!(decks, expected);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_3_unique_card_size_3_deck() {
        let deck: Deck<usize, 3> = new_deck([0, 1, 2]);
        let (count, decks) = collect_new_deck_permutations(deck);

        let expected: HashSet<Deck<usize, 3>> = vec![
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ]
        .into_iter()
        .map(self::new_deck)
        .collect();

        assert_eq!(decks, expected);
        assert_eq!(count, 6);
    }

    #[test]
    fn test_2_unique_card_size_4_deck() {
        let deck: Deck<usize, 4> = new_deck([0, 0, 1, 1]);
        let (count, decks) = collect_new_deck_permutations(deck);

        let expected: HashSet<Deck<usize, 4>> = vec![
            [0, 0, 1, 1],
            [0, 1, 0, 1],
            [0, 1, 1, 0],
            [1, 0, 1, 0],
            [1, 0, 0, 1],
            [1, 1, 0, 0],
        ]
        .into_iter()
        .map(self::new_deck)
        .collect();

        assert_eq!(decks, expected);
        assert_eq!(count, 6);
    }

    #[test]
    fn test_2_unique_card_size_4_deck_2() {
        let deck: Deck<usize, 4> = new_deck([0, 1, 1, 1]);
        let (count, decks) = collect_new_deck_permutations(deck);

        let expected: HashSet<Deck<usize, 4>> =
            vec![[0, 1, 1, 1], [1, 0, 1, 1], [1, 1, 0, 1], [1, 1, 1, 0]]
                .into_iter()
                .map(self::new_deck)
                .collect();

        assert_eq!(decks, expected);
        assert_eq!(count, 4);
    }

    #[test]
    fn test_3_unique_card_size_4_deck() {
        let deck: Deck<usize, 4> = new_deck([0, 1, 1, 2]);
        let (count, decks) = collect_new_deck_permutations(deck);

        let expected: HashSet<Deck<usize, 4>> = vec![
            [0, 1, 1, 2],
            [0, 1, 2, 1],
            [0, 2, 1, 1],
            [1, 0, 1, 2],
            [1, 0, 2, 1],
            [2, 0, 1, 1],
            [1, 1, 0, 2],
            [1, 2, 0, 1],
            [2, 1, 0, 1],
            [1, 1, 2, 0],
            [1, 2, 1, 0],
            [2, 1, 1, 0],
        ]
        .into_iter()
        .map(self::new_deck)
        .collect();

        assert_eq!(decks, expected);
        assert_eq!(count, 12);
    }

    fn new_deck<T, const N: usize>(cards: [T; N]) -> Deck<T, N> {
        Deck::from(cards)
    }

    fn collect_new_deck_permutations<
        T: Default + Copy + Eq + std::hash::Hash + std::fmt::Debug,
        const N: usize,
    >(
        deck: Deck<T, N>,
    ) -> (usize, HashSet<Deck<T, N>>) {
        let mut unique = HashSet::new();
        let mut count = 0;
        let mut permutations = DeckPermutationIterator::new(deck);
        while let Some(next_permutation) = permutations.next() {
            unique.insert(next_permutation.clone());
            count += 1;
        }
        (count, unique)
    }

    #[test]
    fn test_placement_mapping_from_placement_1() {
        let positions = [0, 1];
        let existing_mapping = PlacementMapping([0, 1, 2, 3]);
        let mapping: PlacementMapping<4> =
            PlacementMapping::from_placement_positions_and_mapping(&positions, existing_mapping);
        assert_eq!(mapping.0, [2, 3, 3, 3])
    }

    #[test]
    fn test_placement_mapping_from_placement_2() {
        let positions = [0, 2];
        let existing_mapping = PlacementMapping([0, 1, 2, 3]);
        let mapping: PlacementMapping<4> =
            PlacementMapping::from_placement_positions_and_mapping(&positions, existing_mapping);
        assert_eq!(mapping.0, [1, 3, 3, 3])
    }

    #[test]
    fn test_placement_mapping_from_placement_4() {
        let positions = [1];
        let existing_mapping = PlacementMapping([1, 1]);
        let mapping: PlacementMapping<2> =
            PlacementMapping::from_placement_positions_and_mapping(&positions, existing_mapping);
        assert_eq!(mapping.0, [1, 1])
    }
}
