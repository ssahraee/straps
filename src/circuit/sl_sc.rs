use super::poly::Polynomial;
use std::fmt;

use super::var_set::VarIdx;

static EMPTY_USIZE_ARRAY: [usize; 0] = [];

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum VarSrc {
    Input(usize, usize),
    Random,
    Sum([VarIdx; 2]),
    Product([VarIdx; 2]),
    Not([VarIdx; 1]),
}

impl VarSrc {
    pub fn operands(&self) -> &[VarIdx] {
        match self {
            VarSrc::Sum(ops) | VarSrc::Product(ops) => ops.as_ref(),
            VarSrc::Not(ops) => ops.as_ref(),
            VarSrc::Input(_, _) | VarSrc::Random => EMPTY_USIZE_ARRAY.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var {
    pub src: VarSrc,
    pub output_port: Option<(usize, usize)>,
    pub name: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SlSharedCircuit {
    pub vars: Vec<Var>,
    pub n_shares: usize,
    pub n_input_ports: usize,
    pub n_output_ports: usize,
    pub(crate) anfs: Vec<Polynomial>,
}

impl fmt::Debug for SlSharedCircuit {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SlSharedCircuit")
            .field(
                "vars",
                &format_args!(
                    "{:#?}",
                    self.vars
                        .iter()
                        .enumerate()
                        .map(|x| format!("{:?}", x))
                        .collect::<Vec<_>>()
                ),
            )
            .field("n_output_ports", &self.n_output_ports)
            .field("n_input_ports", &self.n_input_ports)
            .finish()
    }
}

impl SlSharedCircuit {
    pub fn new(
        vars: Vec<Var>,
        n_shares: usize,
        n_input_ports: usize,
        n_output_ports: usize,
    ) -> Self {
        let anfs = build_anfs(&vars);
        Self {
            vars,
            n_shares,
            n_input_ports,
            n_output_ports,
            anfs,
        }
    }
}

fn build_anfs(vars: &[Var]) -> Vec<Polynomial> {
    let mut res: Vec<Polynomial> = Vec::with_capacity(vars.len());
    for (i, var) in vars.iter().enumerate() {
        res.push(match var.src {
            VarSrc::Input(_, _) | VarSrc::Random => Polynomial::from_var(i as u32),
            VarSrc::Not(it) => res[it[0]].clone().not(),
            VarSrc::Sum(it) => res[it[0]].clone() + res[it[1]].clone(),
            VarSrc::Product(it) => res[it[0]].clone() * res[it[1]].clone(),
        });
    }
    return res;
}
