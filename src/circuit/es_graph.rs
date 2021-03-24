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

use super::poly::{Monomial, Polynomial};
use super::sl_sc::{SlSharedCircuit, VarSrc};
use super::var_set::VarIdx;
use itertools::Itertools;
use log::debug;

impl CompGraphWork {
    pub(crate) fn from_circ_probes(
        circ: &SlSharedCircuit,
        probes: impl Iterator<Item = VarIdx>,
    ) -> (Self, Vec<VarIdx>) {
        let mut probes = probes.collect::<Vec<_>>();
        probes.sort();
        let vars: Vec<_> = circ
            .vars
            .iter()
            .map(|var| WorkVar {
                src: var.src.clone(),
                probed: true,
            })
            .collect();
        let successors: Vec<_> = vars.iter().map(|_| Vec::new()).collect();
        let mut res = CompGraphWork {
            vars,
            successors,
            n_input_ports: circ.n_input_ports,
        };
        for (i, var) in circ.vars.iter().enumerate() {
            for op in var.src.operands().iter() {
                assert!(*op < i);
                res.successors[*op].push(i);
            }
        }
        debug!("from_circ_probes: Cleaning graph");
        for p in super::utils::it_complement(0..res.vars.len(), probes.iter().cloned()) {
            res.remove_expr(p);
        }
        debug!("from_circ_probes: Cleaned up graph");
        return (res, probes);
    }
    /// Inputs on which depend the set of probes
    pub(crate) fn inputs(&self) -> Vec<u32> {
        return self
            .remaining_inputs()
            .map(|x| x as u32)
            .collect::<Vec<_>>();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkVar {
    src: VarSrc,
    probed: bool,
}

impl WorkVar {
    fn is_indep_random(&self) -> bool {
        !self.probed && self.src == VarSrc::Random
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompGraphWork {
    n_input_ports: usize,
    vars: Vec<WorkVar>,
    successors: Vec<Vec<VarIdx>>,
}

impl CompGraphWork {
    fn build_anf_erased(&self, var: VarIdx) -> Polynomial {
        match self.vars[var].src {
            VarSrc::Input(_, _) | VarSrc::Random => Polynomial::from_var(var),
            VarSrc::Not(it) => self.build_anf_erased(it[0]).not(),
            VarSrc::Sum(it) => self.build_anf_erased(it[0]) + self.build_anf_erased(it[1]),
            VarSrc::Product(it) => self.build_anf_erased(it[0]) * self.build_anf_erased(it[1]),
        }
    }

    /// Find the first variable that could be pseudo-randomized with `var` as a source
    fn invertible_succ(&self, var: VarIdx) -> Option<VarIdx> {
        let d = self.impdom(var);
        return d.filter(|succ| {
            self.build_anf_erased(*succ)
                .primitive_monomials()
                .into_iter()
                .any(|mon| mon == Monomial::from_var(var))
        });
    }

    fn remove_succ_links_to(&mut self, var: VarIdx, cb_erased: &mut impl FnMut(VarIdx)) {
        debug!("remove_succ_links_to: var: {:?}", var);
        let ops: Vec<VarIdx> = self.vars[var].src.operands().to_vec().to_vec();
        for op in ops.iter() {
            assert!(self.successors[*op].contains(&var));
            self.successors[*op]
                .iter()
                .position(|item| item == &var)
                .map(|i| self.successors[*op].remove(i));
            debug!("removed link {:?} -> {:?}", op, var);
            if !self.vars[*op].probed {
                if self.successors[*op].is_empty() {
                    self.remove_succ_links_to(*op, cb_erased);
                } else if self.vars[*op].src == VarSrc::Random {
                    cb_erased(*op);
                }
            }
        }
    }

    fn remove_expr(&mut self, var: VarIdx) {
        assert!(self.vars[var].probed, "var: {:?}", var);
        debug!(
            "--- remove expr: {:?}, remaining_probes: {:?}",
            var,
            self.vars.iter().positions(|v| v.probed).collect::<Vec<_>>()
        );
        self.vars[var].probed = false;
        if self.successors[var].is_empty() {
            self.remove_succ_links_to(var, &mut |_| {});
        }
    }
    /// replace `var` by an independent random.
    fn erase_var(&mut self, var: VarIdx, stack: &mut Vec<VarIdx>) {
        debug!("erase_var, var: {:?}, stack: {:?}", var, stack);
        if !self.vars[var].probed {
            stack.push(var);
        }
        self.remove_succ_links_to(var, &mut |v| stack.push(v));
        self.vars[var].src = VarSrc::Random;
    }

    pub(crate) fn simplify(&mut self) {
        /* Simplify the graph in a very simple way:
         * while there is a random var that is used only once, and its use is in a sum,
         * replace the sum with a random var.
         */
        // List of random vars to possibly simplify
        debug!("======== simplify =============");
        debug!(
            "probes: {:?}",
            self.vars.iter().positions(|v| v.probed).collect::<Vec<_>>()
        );
        debug!(
            "randoms: {:?}",
            self.vars
                .iter()
                .positions(|v| v.src == VarSrc::Random)
                .collect::<Vec<_>>()
        );
        let mut simplified = true;
        // stask is not sufficient to guarantee optimal simplification, since removal of some
        // values enable simplification while those values are not direct childs of the random
        while simplified {
            simplified = false;
            let mut stack: Vec<_> = self
                .vars
                .iter()
                .enumerate()
                .filter(|(_, var)| var.is_indep_random())
                .map(|(i, _)| i)
                .collect();
            while let Some(i) = stack.pop() {
                debug!(
                    "trying to remove {:?}, successors: {:?}",
                    i, self.successors[i]
                );
                let succ = self.invertible_succ(i);
                debug!("succ: {:?}", succ);
                if let Some(succ) = succ {
                    debug!("erasing {:?} for pred {:?}", succ, i);
                    self.erase_var(succ, &mut stack);
                    simplified = true;
                }
            }
        }
    }

    /// List remaining inputs by browsing the var graph from probes to inputs
    /// excluding erased vars
    /// List remaining inputs by browsing through the anf...
    pub(crate) fn remaining_inputs<'a>(&'a self) -> impl Iterator<Item = VarIdx> + 'a {
        let mut required_inputs: std::collections::BTreeSet<VarIdx> = Default::default();
        for p in self.vars.iter().positions(|v| v.probed) {
            debug!("Remaning inputs of {:?}", p);
            for t in self.build_anf_erased(p).variables::<usize>() {
                debug!("term {:?}", t);
                if let VarSrc::Input(_, _) = self.vars[t].src {
                    required_inputs.insert(t);
                } else {
                    assert!(self.vars[t].src == VarSrc::Random);
                }
            }
        }
        let required_inputs: Vec<_> = required_inputs.into_iter().collect();
        return required_inputs.into_iter();
    }

    /// Immediate post-dominator of var, where sink is a "probed" node and edges are successor
    /// relationships.
    fn impdom(&self, var: VarIdx) -> Option<VarIdx> {
        let mut stack = vec![var];
        // invariant: maxn > i for all i in stack (always except before first pop()
        let mut maxn = var;
        let mut seen = vec![false; maxn];
        // invariant: bound is the min probed var explored and bound >= maxn
        let mut bound = self.vars.len();
        assert!(bound >= maxn);
        while let Some(v) = stack.pop() {
            for succ in self.successors[v].iter().cloned() {
                if self.vars[succ].probed && succ < bound {
                    // either this is a dominator -> ok. otherwise, we are stuck.
                    bound = succ;
                    if maxn > bound {
                        return None; // we a stuck
                    }
                }
                if succ == maxn {
                } else if succ > maxn {
                    if succ > bound {
                        return None;
                    }
                    seen.resize(succ, false);
                    seen[maxn] = true;
                    stack.push(maxn);
                    maxn = succ;
                    assert!(bound >= maxn);
                    assert!(maxn > *stack.iter().max().unwrap());
                } else {
                    assert!(succ < maxn);
                    if !seen[succ] {
                        stack.push(succ);
                        seen[succ] = true;
                        assert!(maxn > *stack.iter().max().unwrap());
                    }
                }
            }
        }
        // stack is empty, maxn is the idom of start, and no probes between start and maxn
        assert!((maxn != var) || self.successors[var].is_empty());
        if maxn == var {
            return None;
        }
        return Some(maxn);
    }
}
