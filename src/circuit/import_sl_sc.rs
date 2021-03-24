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

use super::sl_sc::{SlSharedCircuit, Var, VarSrc};
use super::utils;
fn check_no_io<'a>(names: impl Iterator<Item = &'a str>) -> Result<(), String> {
    names
        .filter_map(|name| {
            if name.starts_with("input") || name.starts_with("output") {
                Some(Err(name.to_owned()))
            } else {
                None
            }
        })
        .collect()
}

fn check_completeness_sharings(
    vars: &[Var],
    n_shares: usize,
    n_ports: usize,
    kind: &str,
    extract_port_share: impl Fn(&Var) -> Option<(usize, usize)>,
) -> Result<(), String> {
    let mut shares = vec![vec![false; n_shares]; n_ports];
    for v in vars.iter() {
        if let Some((port, share)) = extract_port_share(v) {
            if shares[port][share] {
                return Err(format!("Duplicate {} share ({}, {})", kind, port, share));
            }
            shares[port][share] = true;
        }
    }
    let missing_shares: Vec<_> = shares
        .iter()
        .enumerate()
        .map(|(ip, p)| {
            p.iter()
                .enumerate()
                .filter(|&(_, ref b)| !**b)
                .map(move |(is, _)| (ip, is))
        })
        .flatten()
        .collect();
    if !missing_shares.is_empty() {
        return Err(format!("Missing {}_shares {:?}", kind, missing_shares));
    } else {
        return Ok(());
    }
}

pub fn new_sl_sc(
    vars: Vec<Var>,
    n_shares: usize,
    n_input_ports: usize,
    n_output_ports: usize,
) -> Result<SlSharedCircuit, String> {
    // check unicity of names
    if !utils::is_unique(vars.iter().map(|v| &v.name)) {
        return Err("Non unique names".to_owned());
    }
    // check names not input/output
    check_no_io(vars.iter().map(|v| v.name.as_str()))?;
    // * ordering of variables
    for (i, v) in vars.iter().enumerate() {
        for op in v.src.operands().iter() {
            if *op >= i {
                return Err(format!("Invalid op sorting, {} {}", i, op));
            }
        }
    }
    // check completeness and unicity of (port, idx) pairs for inputs and outputs
    check_completeness_sharings(&vars, n_shares, n_input_ports, "input", |v| {
        if let VarSrc::Input(port, share) = v.src {
            Some((port, share))
        } else {
            None
        }
    })?;
    check_completeness_sharings(&vars, n_shares, n_output_ports, "output", |v| {
        v.output_port.clone()
    })?;
    return Ok(SlSharedCircuit::new(
        vars,
        n_shares,
        n_input_ports,
        n_output_ports,
    ));
}
