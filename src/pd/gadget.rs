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

use crate::circuit;
use itertools::Itertools;

pub(crate) trait Gadget {
    fn n_outputs(&self) -> usize;
    fn n_inputs(&self) -> usize;
    fn max_n_probes(&self) -> u32;
    fn n_shares(&self) -> usize;
    fn n_input_sharings(&self) -> usize;
    fn n_output_sharings(&self) -> usize;
    fn n_pp(&self) -> usize;
    /// Shall be sorted in decreasing order.
    fn pp_maxp(&self) -> &[u32];
    fn sim_probes(
        &self,
        outputs: impl IntoIterator<Item = usize>,
        probes: impl IntoIterator<Item = usize>,
    ) -> Vec<usize>;
}

#[derive(Debug, Clone)]
pub(crate) struct SimGadget {
    circuit: circuit::SlSharedCircuit,
    max_probes: Vec<u32>,
    probes2vars: Vec<usize>,
    output_vars: Vec<usize>,
}

impl SimGadget {
    pub(crate) fn new(circuit: circuit::SlSharedCircuit, uc2maxp: impl Fn(u32) -> u32) -> Self {
        let mut var_use_count = vec![0; circuit.vars.len()];
        for var in circuit.vars.iter() {
            match var.src {
                circuit::VarSrc::Sum([v1, v2]) | circuit::VarSrc::Product([v1, v2]) => {
                    var_use_count[v1] += 1;
                    var_use_count[v2] += 1;
                }
                circuit::VarSrc::Not([v]) => {
                    var_use_count[v] += 1;
                }
                circuit::VarSrc::Input(_, _) | circuit::VarSrc::Random => {}
            }
        }
        let mut sorted_probe_vars = var_use_count
            .into_iter()
            .enumerate()
            .filter(|(_, use_count)| *use_count != 0)
            .collect::<Vec<_>>();
        sorted_probe_vars.sort_unstable_by(|(_, uc1), (_, uc2)| uc2.cmp(uc1));
        let probes2vars = sorted_probe_vars
            .iter()
            .map(|(v, _)| *v)
            .collect::<Vec<_>>();
        let max_probes = sorted_probe_vars
            .iter()
            .map(|(_, uc)| uc2maxp(*uc))
            .collect::<Vec<_>>();
        let mut output_vars = circuit
            .vars
            .iter()
            .enumerate()
            .filter_map(|(i, var)| var.output_port.map(|port| (port, i)))
            .collect::<Vec<_>>();
        output_vars.sort_unstable();
        let output_vars = output_vars.into_iter().map(|(_, i)| i).collect::<Vec<_>>();
        return Self {
            circuit,
            max_probes,
            probes2vars,
            output_vars,
        };
    }
}

impl Gadget for SimGadget {
    fn n_outputs(&self) -> usize {
        self.n_output_sharings() * self.n_shares()
    }
    fn n_inputs(&self) -> usize {
        self.n_input_sharings() * self.n_shares()
    }
    fn max_n_probes(&self) -> u32 {
        self.max_probes.iter().copied().sum::<u32>()
    }
    fn n_shares(&self) -> usize {
        self.circuit.n_shares
    }
    fn n_input_sharings(&self) -> usize {
        self.circuit.n_input_ports
    }
    fn n_output_sharings(&self) -> usize {
        self.circuit.n_output_ports
    }
    fn n_pp(&self) -> usize {
        self.max_probes.len()
    }
    fn pp_maxp(&self) -> &[u32] {
        &self.max_probes
    }
    /// Numbering of inputs and outputs corresponds to the concatenation of the sharings.
    fn sim_probes(
        &self,
        outputs: impl IntoIterator<Item = usize>,
        probes: impl IntoIterator<Item = usize>,
    ) -> Vec<usize> {
        let mut var_mask = vec![false; self.circuit.vars.len()];
        for output in outputs {
            var_mask[self.output_vars[output]] = true;
        }
        for probe in probes {
            var_mask[self.probes2vars[probe]] = true;
        }
        let probed_vars = var_mask
            .into_iter()
            .positions(|x| x)
            .map(|i| i as u32)
            .collect::<Vec<_>>();
        let req_inputs: Vec<u32> = circuit::sim_set(&self.circuit, probed_vars);
        let input_offset = req_inputs
            .into_iter()
            .map(|i| match self.circuit.vars[i as usize].src {
                circuit::VarSrc::Input(port, share) => port * self.n_shares() + share,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        return input_offset;
    }
}
