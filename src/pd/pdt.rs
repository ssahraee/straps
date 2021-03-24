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

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone)]
pub struct ProbeDistribution<W: Clone + Eq + Hash> {
    n: u32,
    pub wires: Vec<W>,
    // reverse map of `wires`
    wire2idx: HashMap<W, u32>,
    pub distr: ndarray::Array1<f64>,
}

impl<W: Clone + Eq + Hash> ProbeDistribution<W> {
    pub fn from_wires(wires: Vec<W>) -> Self {
        let mut distr = ndarray::Array1::zeros(1 << wires.len());
        distr[0] = 1.0;
        return Self::from_wires_distr(wires, distr);
    }

    pub fn from_wires_distr(wires: Vec<W>, distr: impl Into<ndarray::Array1<f64>>) -> Self {
        let n = wires.len() as u32;
        let wire2idx = wires
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, w)| (w, i as u32))
            .collect::<HashMap<_, _>>();
        let distr = distr.into();
        return Self {
            n,
            wires,
            wire2idx,
            distr,
        };
    }

    fn swap(&mut self, i: u32, j: u32) {
        // ensure i < j
        if i == j {
            return;
        }
        let (i, j) = if i < j { (i, j) } else { (j, i) };
        // update wire <-> idx mapping
        *self.wire2idx.get_mut(&self.wires[i as usize]).unwrap() = j;
        *self.wire2idx.get_mut(&self.wires[j as usize]).unwrap() = i;
        self.wires.swap(i as usize, j as usize);
        // Let b_{n-1}...b_0 be the bit-representation of an index into the
        // distributions.
        // For all indexes b_{n-1}...b_j...b_i...b_0, we want to swap the
        // corresponding distribution element with the element at index
        // b_{n-1}...b_i...b_j...b_0.
        // We can to this only for b_i != b_j (otherwise the two indexes are the
        // same), and we want to swap only once, therefore we always take b_i=1
        // and b_j=0.
        // We split the index into three parts, which are collections of bits:
        // index is x b_j y b_i z.
        // Or equivalently, index is x + b_j + y + b_i + z where
        // z is in the set {k*2^0: k in [0, 2^i-1]}
        //   (from 0 to 2^i-1 by step of 1)
        // y is in the set {k*2^(i+1): k in [0, 2^(j-i-1)-1]}
        //   (from 0 to 2^j-1 by step of 2^(i+1))
        // x is in the set {k*2^(j+1): k in [0, 2^(n-j-1)-1]}
        //   (from 0 to 2^n-1 by step of 2^(j+1))
        let mut x = 0;
        while x < (1 << self.n) {
            let mut y = 0;
            while y < (1 << j) {
                for z in 0..(1 << i) {
                    let base_idx = x + y + z;
                    let idx0 = base_idx + (1 << i);
                    let idx1 = base_idx + (1 << j);
                    self.distr.swap(idx0, idx1);
                }
                y += 1 << (i + 1)
            }
            x += 1 << (j + 1);
        }
    }

    pub fn apply_op(
        &mut self,
        inputs: Vec<W>,
        outputs: Vec<W>,
        pdt: &ndarray::Array2<f64>,
    ) -> Self {
        let in_chunk = 1 << inputs.len();
        let out_chunk = 1 << outputs.len();
        assert_eq!(pdt.shape(), &[in_chunk, out_chunk]);
        for in_ in inputs.iter() {
            if !outputs.contains(in_) {
                assert!(!self.wires.contains(in_));
                // TODO handle this case, currently it is forbidden (cases: merge
                // two wires, and keep output)
            }
        }
        // Put outputs as first positions
        for (i, out) in outputs.iter().enumerate() {
            self.swap(self.wire2idx[out], i as u32);
        }
        // Now multiply the distribution by I_{2^(n-len(outputs)) \otimes pdt.
        // This is equivalent to multiplying all subsets of size 2^len(outputs)
        // of the distributions by pdt.
        let new_n = self.n + inputs.len() as u32 - outputs.len() as u32;
        let mut new_distr = ndarray::Array1::zeros(1 << new_n);
        for chunk in 0..(1 << self.n - outputs.len() as u32) {
            let i_old = chunk * out_chunk;
            let i_new = chunk * in_chunk;
            new_distr
                .slice_mut(ndarray::s![i_new..i_new + in_chunk])
                .assign(
                    &pdt.view()
                        .dot(&self.distr.slice(ndarray::s![i_old..i_old + out_chunk])),
                );
        }
        let new_wires = inputs
            .iter()
            .cloned()
            .chain(self.wires.iter().cloned().skip(outputs.len()))
            .collect::<Vec<_>>();
        Self::from_wires_distr(new_wires, new_distr)
    }

    pub fn leak_wire(&mut self, var: W, p: f64) -> Self {
        self.apply_op(
            vec![var.clone()],
            vec![var.clone()],
            &ndarray::array![[1.0 - p, 0.0], [p, 1.0],],
        )
    }
    pub fn bin_op(&mut self, dest: W, src1: W, src2: W, p: f64) -> Self {
        self.apply_op(
            vec![src1, src2],
            vec![dest],
            &ndarray::array![
                [(1.0 - p) * (1.0 - p), 0.0],
                [p * (1.0 - p), 0.0],
                [p * (1.0 - p), 0.0],
                [p * p, 1.0],
            ],
        )
    }

    pub fn split_wire(&mut self, src: W, dest1: W, dest2: W) -> Self {
        self.apply_op(
            vec![src],
            vec![dest1, dest2],
            &ndarray::array![[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 1.0, 1.0]],
        )
    }

    pub fn get_distr(&self) -> &ndarray::Array1<f64> {
        &self.distr
    }

    pub fn wire_idx(&self, wire: &W) -> u32 {
        self.wire2idx[&wire]
    }
}
