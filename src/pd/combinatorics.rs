// STRAPS - Statistical Testing of RAndom Probing Security
// Copyright (C) 2021 UCLouvain
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use ndarray::s;

/// Iterator over all combinations of k elements of a set {0,...,max-1}, such that the sum of the
/// weights of the elements is at least min_weight.
#[derive(Debug, Clone)]
pub(crate) struct MWCombinations<'a> {
    weights_sorted: &'a [u32],
    min_weight: u32,
    first: bool,
    indices: Vec<usize>,
    indices_final: Vec<usize>,
}
impl<'a> MWCombinations<'a> {
    /// Build a MWCombinations.
    ///
    /// The weights of an element is obtained by indexing weights_sorted with the element.
    /// `weights_sorted` must be sorted in decreasing order.
    pub(crate) fn new(weights_sorted: &'a [u32], k: usize, min_weight: u32) -> Self {
        Self {
            weights_sorted,
            min_weight,
            first: true,
            indices: (0..k).collect(),
            indices_final: last_combination(k, weights_sorted, min_weight),
        }
    }
}
impl<'a> Iterator for MWCombinations<'a> {
    type Item = Vec<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            if self.indices.len() > self.weights_sorted.len()
                || self.weights_sorted[..self.indices.len()]
                    .iter()
                    .copied()
                    .sum::<u32>()
                    < self.min_weight
            {
                return None;
            }
            self.first = false;
        } else if self.indices.is_empty() {
            // If we select 0 elements, then there is only one selection.
            return None;
        } else {
            // Can we advance the i-th index and reset all next indices ?
            let may_advance = |i| {
                // first condition: not at the end of the vec
                // second condition: is cheap enough
                self.indices[i] < i + self.weights_sorted.len() - self.indices.len()
                    && (0..i)
                        .map(|j| self.indices[j])
                        .chain(((self.indices[i] + 1)..).take(self.indices.len() - i))
                        .map(|idx| self.weights_sorted[idx])
                        .sum::<u32>()
                        >= self.min_weight
            };
            // Scan from the end, looking for an index to increment
            let mut i = self.indices.len() - 1;
            while !may_advance(i) {
                if i > 0 {
                    i -= 1;
                } else {
                    // Reached the last combination
                    return None;
                }
            }
            // Increment index, and reset the ones to its right
            self.indices[i] += 1;
            for j in i + 1..self.indices.len() {
                self.indices[j] = self.indices[j - 1] + 1;
            }
        }
        debug_assert!(
            self.indices
                .iter()
                .map(|i| self.weights_sorted[*i])
                .sum::<u32>()
                >= self.min_weight
        );
        Some(self.indices.clone())
    }
}

fn last_combination(k: usize, weights_sorted: &[u32], min_weight: u32) -> Vec<usize> {
    let mut res = (0..k).collect::<Vec<usize>>();
    while move_fw_combination(res.as_mut_slice(), weights_sorted, min_weight).is_ok() {}
    return res;
}

fn move_fw_combination(
    indices: &mut [usize],
    weights_sorted: &[u32],
    min_weight: u32,
) -> Result<usize, ()> {
    for idx_to_move in 0..indices.len() {
        let sunk_weights = (0..idx_to_move)
            .map(|i| weights_sorted[indices[i]])
            .sum::<u32>();
        let moved = indices[idx_to_move] + 1;
        let n_to_move = indices.len() - idx_to_move;
        let moved_ub = moved + n_to_move;
        if moved_ub > weights_sorted.len() {
            return Err(());
        }
        if sunk_weights + (moved..moved_ub).map(|i| weights_sorted[i]).sum::<u32>() >= min_weight {
            indices[idx_to_move..]
                .iter_mut()
                .enumerate()
                .for_each(|(i, idx)| {
                    *idx = moved + i;
                });
            return Ok(idx_to_move);
        }
    }
    return Err(());
}

/// Returns an array `res` of length lim_selected, where `res[i]` is the number of ways of
/// selecting `i` items in the multiset represented by use_counts, such that each distinct element
/// of the multiset is selected at least once.
/// The number of distinct elements in the multiset if the length of use_counts and the number of
/// repetitions of each element is the corresponding value in use_counts.
pub(crate) fn count_selections(
    range_selected: std::ops::Range<usize>,
    use_counts: &[usize],
) -> ndarray::Array1<u64> {
    let mut scratch_space = ndarray::Array1::zeros((range_selected.end,));
    let mut tmp_scratch_space = ndarray::Array1::zeros((range_selected.end,));
    if range_selected.end > 0 {
        scratch_space[0] = 1;
    }
    let tot_use = use_counts.iter().copied().sum::<usize>() as isize;
    let mut sub_range_start = (range_selected.start as isize) - tot_use;
    let mut sub_range_end = (range_selected.end as isize) - (use_counts.len() as isize);
    for count in use_counts {
        if *count == 0 {
            return ndarray::Array1::zeros((range_selected.len(),));
        }
        let sr_start = std::cmp::max(sub_range_start, 0) as usize;
        sub_range_start += *count as isize;
        sub_range_end += 1;
        let next_sr_start = std::cmp::max(sub_range_start, 0) as usize;
        let next_sr_end = std::cmp::max(sub_range_end, 0) as usize;
        let mut alpha = *count as u64;
        for i in 1..std::cmp::min(*count + 1, next_sr_end - sr_start) {
            let first_dest_item = std::cmp::max(next_sr_start, sr_start + i);
            let first_src_item = first_dest_item - i;
            // slice length: this ensures smaller than sub_range.len()
            let length = next_sr_end - first_dest_item;
            tmp_scratch_space
                .slice_mut(s![first_dest_item..(first_dest_item + length)])
                .scaled_add(
                    alpha,
                    &scratch_space.slice(s![first_src_item..(first_src_item + length)]),
                );
            debug_assert_eq!(
                alpha as f64,
                statrs::function::factorial::binomial(*count as u64, i as u64)
            );
            alpha *= (*count - i) as u64;
            alpha /= (i + 1) as u64;
        }
        scratch_space.assign(&tmp_scratch_space);
        tmp_scratch_space.fill(0);
    }
    return scratch_space.slice(s![range_selected]).to_owned();
}

#[test]
fn test_count_selections() {
    let cases = vec![
        (2, vec![0, 1], vec![0, 0]),
        (2, vec![], vec![1, 0]),
        (3, vec![1], vec![0, 1, 0]),
        (4, vec![2], vec![0, 2, 1, 0]),
        (4, vec![1, 1], vec![0, 0, 1, 0]),
        (6, vec![1, 2], vec![0, 0, 2, 1, 0, 0]),
        (6, vec![1, 3], vec![0, 0, 3, 3, 1, 0]),
        (7, vec![1, 1, 3], vec![0, 0, 0, 3, 3, 1, 0]),
        (8, vec![2, 1, 3], vec![0, 0, 0, 6, 2 * 3 + 3, 2 + 3, 1, 0]),
        (5, vec![2, 1, 3], vec![0, 0, 0, 6, 2 * 3 + 3]),
        (7, vec![5], vec![0, 5, 10, 10, 5, 1, 0]),
        (1, vec![5], vec![0]),
        (0, vec![5], vec![]),
    ];
    for (res_len, problem, solution) in cases {
        assert_eq!(
            count_selections(res_len, &problem).to_vec(),
            solution,
            "res_len: {}, problem: {:?}",
            res_len,
            problem
        );
        for i in 0..res_len {
            for j in i..res_len {
                assert_eq!(
                    count_selections2(i..j, &problem).as_slice().unwrap(),
                    &solution[i..j],
                    "count2, i: {}, j: {}, problem: {:?}, solution: {:?}",
                    i,
                    j,
                    problem,
                    solution
                );
                assert_eq!(
                    count_selections3(i..j, &problem).as_slice().unwrap(),
                    &solution[i..j],
                    "count3, i: {}, j: {}, problem: {:?}, solution: {:?}",
                    i,
                    j,
                    problem,
                    solution
                );
            }
        }
    }
}
