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

use super::{new_sl_sc, SlSharedCircuit, Var, VarSrc};

fn collect_single<T>(n: usize, f: impl FnMut(usize) -> T) -> Vec<T> {
    (0..n).map(f).collect()
}
fn collect_all<T>(n: usize, mut f: impl FnMut(usize, usize) -> T) -> Vec<Vec<T>> {
    collect_single(n, |i| collect_single(n, |j| f(i, j)))
}

pub fn build_isw(n: usize) -> SlSharedCircuit {
    let mut vars = vec![];
    let mut new_var = |v| {
        vars.push(v);
        vars.len() - 1
    };
    let input_x = collect_single(n, |i| {
        new_var(Var {
            src: VarSrc::Input(0, i),
            output_port: None,
            name: format!("x_{}", i),
        })
    });
    let input_y = collect_single(n, |i| {
        new_var(Var {
            src: VarSrc::Input(1, i),
            output_port: None,
            name: format!("y_{}", i),
        })
    });
    let randoms = collect_all(n, |i, j| {
        new_var(Var {
            src: VarSrc::Random,
            output_port: None,
            name: format!("r_{}_{}", i, j),
        })
    });
    if n == 1 {
        new_var(Var {
            src: VarSrc::Product([input_x[0], input_y[0]]),
            output_port: Some((0, 0)),
            name: "o_0".to_owned(),
        });
    } else {
        let products = collect_all(n, |i, j| {
            new_var(Var {
                src: VarSrc::Product([input_x[i], input_y[j]]),
                output_port: None,
                name: format!("p_{}_{}", i, j),
            })
        });
        let ref_prod = collect_all(n, |i, j| {
            if i == j {
                products[i][j]
            } else {
                new_var(Var {
                    src: VarSrc::Sum([
                        products[i][j],
                        if i < j { randoms[j][i] } else { randoms[i][j] },
                    ]),
                    output_port: None,
                    name: format!("t_{}_{}", i, j),
                })
            }
        });
        let _cumsums = collect_single(n, |i| {
            (1..n).fold(ref_prod[i][0], |acc, j| {
                new_var(Var {
                    src: VarSrc::Sum([acc, ref_prod[i][j]]),
                    output_port: if j == n - 1 { Some((0, i)) } else { None },
                    name: format!("c_{}_{}", i, j),
                })
            })
        });
    }
    let circuit = new_sl_sc(vars, n, 2, 1).expect("Bad ISW");
    return circuit;
}
