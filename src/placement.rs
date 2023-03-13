#[derive(Debug, Copy, Clone)]
pub struct PlacementIterator<const N: usize> {
    k: usize,
    n: usize,
    c: [usize; N],
    j: usize,
    finished: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct Placement<const N: usize>([usize; N], usize);

impl<const N: usize> PlacementIterator<N> {
    pub fn new(n: usize, k: usize) -> Self {
        if k > n {
            panic!("k = {} is too large, max value is n = {}", k, n);
        }
        if n > N {
            panic!("n = {} is too large, max value is N = {}", n, N);
        }
        let mut c = [0; N];
        for i in (0..k).into_iter() {
            c[i] = i;
        }
        if k < n {
            c[k] = n;
        }
        Self {
            k,
            n,
            c,
            j: 0,
            finished: false,
        }
    }

    pub fn reset(&mut self) {
        if !self.finished {
            panic!("Trying to reset unfinished iterator");
        }
        *self = Self::new(self.n, self.k);
    }

    fn update_combination(&mut self) -> bool {
        if self.finished {
            return false;
        }
        if self.k == self.n {
            self.finished = true;
            return false;
        }
        if self.k == 1 {
            self.c[0] += 1;
            if self.c[0] == self.c[1] {
                self.finished = true;
            }
            return true;
        }
        if self.k % 2 == 0 {
            if self.c[0] > 0 {
                self.c[0] -= 1;
            } else {
                self.j = 1;
                self.try_to_increase();
            }
        } else {
            if self.c[0] + 1 < self.c[1] {
                self.c[0] += 1;
            } else {
                self.j = 1;
                self.try_to_decrease();
            }
        }
        true
    }

    fn try_to_increase(&mut self) {
        if self.c[self.j] + 1 < self.c[self.j + 1] {
            self.c[self.j - 1] = self.c[self.j];
            self.c[self.j] += 1;
        } else {
            self.j += 1;
            if self.j + 1 <= self.k {
                self.try_to_decrease();
            } else {
                self.finished = true;
            }
        }
    }

    fn try_to_decrease(&mut self) {
        if self.c[self.j] >= self.j + 1 {
            self.c[self.j] = self.c[self.j - 1];
            self.c[self.j - 1] = self.j - 1;
        } else {
            self.j += 1;
            self.try_to_increase();
        }
    }
}

impl<const N: usize> Default for PlacementIterator<N> {
    fn default() -> Self {
        Self::new(N, N)
    }
}

impl<const N: usize> Iterator for PlacementIterator<N> {
    type Item = Placement<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let next_combination = Placement(self.c, self.k);
        self.update_combination();

        Some(next_combination)
    }
}

impl<const N: usize> Placement<N> {
    pub fn positions(&self) -> &[usize] {
        &self.0[0..self.1]
    }
}

impl<const N: usize> Default for Placement<N> {
    fn default() -> Self {
        let mut positions = [0; N];
        for i in 0..N {
            positions[i] = i;
        }
        Self(positions, N)
    }
}

#[cfg(test)]
mod tests {
    use super::PlacementIterator;
    use std::collections::HashSet;

    #[test]
    fn placements_n_1_k_1() {
        let (all_results, unique_results) = generate_placements::<1>(1, 1);
        let expected = vec![0b1].into_iter().collect();

        assert_eq!(all_results.len(), 1);
        assert_eq!(unique_results, expected);
    }

    #[test]
    fn placements_n_2_k_1() {
        let (all_results, unique_results) = generate_placements::<2>(2, 1);
        let expected = vec![0b01, 0b10].into_iter().collect();

        assert_eq!(all_results.len(), 2);
        assert_eq!(unique_results, expected);
    }

    #[test]
    fn placements_n_1_k_1_capacity_2() {
        let (all_results, unique_results) = generate_placements::<2>(1, 1);
        let expected = vec![0b1].into_iter().collect();

        assert_eq!(all_results.len(), 1);
        assert_eq!(unique_results, expected);
    }

    #[test]
    fn placements_n_2_k_2() {
        let (all_results, unique_results) = generate_placements::<2>(2, 2);
        let expected = vec![0b11].into_iter().collect();

        assert_eq!(all_results.len(), 1);
        assert_eq!(unique_results, expected);
    }

    #[test]
    fn placements_n_3_k_2() {
        let (all_results, unique_results) = generate_placements::<3>(3, 2);
        let expected = vec![0b011, 0b101, 0b110].into_iter().collect();

        assert_eq!(all_results.len(), 3);
        assert_eq!(unique_results, expected);
    }

    #[test]
    fn placements_n_3_k_1() {
        let (all_results, unique_results) = generate_placements::<3>(3, 1);
        let expected = vec![0b100, 0b010, 0b001].into_iter().collect();

        assert_eq!(all_results.len(), 3);
        assert_eq!(unique_results, expected);
    }

    #[test]
    fn placements_n_4_k_2() {
        let (all_results, unique_results) = generate_placements::<4>(4, 2);
        let expected = vec![0b0011, 0b0101, 0b0110, 0b1010, 0b1100, 0b1001]
            .into_iter()
            .collect();

        assert_eq!(all_results.len(), 6);
        assert_eq!(unique_results, expected);
    }

    #[test]
    fn placements_n_4_k_3() {
        let (all_results, unique_results) = generate_placements::<4>(4, 3);
        let expected = vec![0b1110, 0b1101, 0b1011, 0b0111].into_iter().collect();

        assert_eq!(all_results.len(), 4);
        assert_eq!(unique_results, expected);
    }

    #[test]
    fn placements_n_6_k_3() {
        let (all_results, unique_results) = generate_placements::<6>(6, 3);
        let expected = vec![
            0b000111, 0b001011, 0b001101, 0b001110, 0b010110, 0b010101, 0b010011, 0b011001,
            0b011010, 0b011100, 0b101100, 0b101001, 0b101010, 0b100110, 0b100101, 0b100011,
            0b110001, 0b110010, 0b110100, 0b111000,
        ]
        .into_iter()
        .collect();

        assert_eq!(unique_results, expected);
        assert_eq!(all_results.len(), 20);
    }

    fn generate_placements<const N: usize>(n: usize, k: usize) -> (Vec<usize>, HashSet<usize>) {
        let all: Vec<usize> = PlacementIterator::<N>::new(n, k)
            .map(|gc| {
                let mut result = 0;
                for element_position in gc.positions() {
                    result = result | (1 << element_position);
                }
                result
            })
            .collect();
        let unique = all.iter().cloned().collect();
        (all, unique)
    }
}
