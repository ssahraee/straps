// STRAPS - Statistical Testing of RAndom Probing Security
// Copyright (C) 2021 - 2024 UCLouvain, Gaëtan Cassiers
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

use numpy::{PyArray2, PyArray3, PyArrayMethods, ToPyArray};
use pyo3::prelude::*;
use std::convert::TryInto;

pub mod circuit;
pub mod pd;

macro_rules! py_type_wrapper {
    ($inner_type:ty,$wrapper_name:ident) => {
        #[pyclass]
        struct $wrapper_name {
            inner: $inner_type,
        }
        impl std::convert::From<$inner_type> for $wrapper_name {
            fn from(inner: $inner_type) -> Self {
                Self { inner }
            }
        }
        impl std::convert::From<$wrapper_name> for $inner_type {
            fn from(wrapper: $wrapper_name) -> Self {
                wrapper.inner
            }
        }
    };
}

py_type_wrapper!(pd::CntSim<pd::SimGadget>, PyCntSim);
py_type_wrapper!(pd::CntSimSt, PyCntSimSt);
py_type_wrapper!(pd::SampleRes, PySampleRes);
py_type_wrapper!(pd::GPdt, PyGPdt);
py_type_wrapper!(ndarray::Array2<f64>, PyPDT);
py_type_wrapper!(circuit::SlSharedCircuit, PyCompGraph);
py_type_wrapper!(
    std::sync::RwLock<pd::ProbeDistribution<String>>,
    PyProbeDistribution
);

#[pymodule]
fn _straps_ext(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCompGraph>()?;
    m.add_class::<PyCntSim>()?;
    m.add_class::<PyCntSimSt>()?;
    m.add_class::<PyProbeDistribution>()?;
    m.add_class::<PyPDT>()?;
    m.add_class::<PyGPdt>()?;
    Ok(())
}

#[derive(Debug, Clone)]
struct SErr(String);
impl std::convert::From<SErr> for PyErr {
    fn from(err: SErr) -> PyErr {
        pyo3::exceptions::PyValueError::new_err(err.0)
    }
}
impl<'a> std::convert::From<&'a str> for SErr {
    fn from(err: &'a str) -> Self {
        Self(err.into())
    }
}

#[pymethods]
impl PyCompGraph {
    #[new]
    fn new(
        vars: Vec<(u32, Vec<usize>, String)>,
        in_ports: Vec<Option<(usize, usize)>>,
        out_ports: Vec<Option<(usize, usize)>>,
        n_shares: usize,
        n_input_ports: usize,
        n_output_ports: usize,
    ) -> PyResult<PyCompGraph> {
        let vars = vecs2graph(vars, in_ports, out_ports)?;
        let comp_graph = circuit::new_sl_sc(vars, n_shares, n_input_ports, n_output_ports)
            .map_err(|e| SErr(e))?;
        return Ok(comp_graph.into());
    }
    fn sim_set(&self, probes: Vec<u32>) -> Vec<u32> {
        circuit::sim_set(&self.inner, probes)
    }

    fn cnt_sim(&self, use_copy: bool) -> PyCntSim {
        let uc2mp = |uc| if use_copy { 2 * uc - 1 } else { uc };
        pd::CntSim::new(pd::SimGadget::new(self.inner.clone(), uc2mp)).into()
    }
    fn output_ports(&self) -> Vec<Option<(usize, usize)>> {
        self.inner.vars.iter().map(|v| v.output_port).collect()
    }
    fn input_ports(&self) -> Vec<Option<(usize, usize)>> {
        self.inner
            .vars
            .iter()
            .map(|v| {
                if let circuit::VarSrc::Input(p, ix) = v.src {
                    Some((p, ix))
                } else {
                    None
                }
            })
            .collect()
    }
    fn name(&self, i: usize) -> String {
        self.inner.vars[i].name.clone()
    }
    fn n_vars(&self) -> usize {
        self.inner.vars.len()
    }
    fn n_input_ports(&self) -> usize {
        self.inner.n_input_ports
    }
    fn n_output_ports(&self) -> usize {
        self.inner.n_output_ports
    }
    fn n_shares(&self) -> usize {
        self.inner.n_shares
    }
    #[classattr]
    const VAR_KIND_INPUT: u32 = 0;
    #[classattr]
    const VAR_KIND_RANDOM: u32 = 1;
    #[classattr]
    const VAR_KIND_SUM: u32 = 2;
    #[classattr]
    const VAR_KIND_PRODUCT: u32 = 3;
    #[classattr]
    const VAR_KIND_NOT: u32 = 4;
    fn var_kind(&self, i: usize) -> u32 {
        match self.inner.vars[i].src {
            circuit::VarSrc::Input(_, _) => Self::VAR_KIND_INPUT,
            circuit::VarSrc::Random => Self::VAR_KIND_RANDOM,
            circuit::VarSrc::Sum(_) => Self::VAR_KIND_SUM,
            circuit::VarSrc::Product(_) => Self::VAR_KIND_PRODUCT,
            circuit::VarSrc::Not(_) => Self::VAR_KIND_NOT,
        }
    }
    fn var_inputs(&self, i: usize) -> Vec<usize> {
        self.inner.vars[i].src.operands().to_owned()
    }
    fn n_inputs(&self) -> usize {
        self.inner.n_shares * self.inner.n_input_ports
    }
    fn n_outputs(&self) -> usize {
        self.inner.n_shares * self.inner.n_output_ports
    }
}

#[pymethods]
impl PySampleRes {
    fn n_probes(&self) -> usize {
        self.inner.n_probes
    }
    fn counts(&self) -> Vec<u64> {
        self.inner.counts.clone()
    }
    fn exhaustive(&self) -> bool {
        self.inner.exhaustive
    }
}
#[pymethods]
impl PyCntSim {
    fn run_sampling<'p>(&self, py: Python<'p>, n_s_max: u32, suff_thresh: u32) -> PyCntSimSt {
        py.allow_threads(|| {
            let firestorm_dir = std::env::var("STRAPS_FIRESTORM_DIR");
            if firestorm_dir.is_ok() {
                firestorm::clear();
            }
            let res = self.inner.run_sampling(n_s_max, suff_thresh).into();
            if let Ok(firestorm_dir) = firestorm_dir {
                firestorm::save(firestorm_dir).unwrap();
            }
            res
        })
    }
    fn probe_output<'p>(
        &self,
        py: Python<'p>,
        n_s_max: u32,
        suff_thresh: u32,
        output_id: usize,
    ) -> Vec<PySampleRes> {
        py.allow_threads(|| {
            let style = indicatif::ProgressStyle::default_bar()
                .template("{msg} [{bar:40}] {pos}/{len} [{elapsed_precise}>{eta_precise}]");
            pd::multiprogress::MultiProgressConfig::new(1, style).run(|mp| {
                let sub_progress = mp.sub(0);
                self.inner
                    .probe_output(output_id, n_s_max, suff_thresh, sub_progress)
                    .map(|x| x.into())
                    .collect::<Vec<_>>()
            })
        })
    }
}

#[pymethods]
impl PyCntSimSt {
    #[new]
    fn new(counts: &Bound<'_, PyArray3<u64>>, exhaustive: &Bound<'_, PyArray2<bool>>) -> Self {
        pd::CntSimSt {
            cnt: counts.to_owned_array(),
            exhaustive: exhaustive.to_owned_array(),
        }
        .into()
    }
    fn estimate<'p>(&self, py: Python<'p>) -> PyGPdt {
        py.allow_threads(|| self.inner.estimate().into())
    }
    fn ub<'p>(&self, py: Python<'p>, err: f64, cum_tr: bool) -> PyGPdt {
        py.allow_threads(|| self.inner.ub(err, cum_tr).into())
    }
    fn lb<'p>(&self, py: Python<'p>, err: f64, cum_tr: bool) -> PyGPdt {
        py.allow_threads(|| self.inner.lb(err, cum_tr).into())
    }
    fn to_array<'p>(&self, py: Python<'p>) -> Bound<'p, PyArray3<u64>> {
        PyArray3::from_array_bound(py, &self.inner.cnt)
    }
    fn exhaustive<'p>(&self, py: Python<'p>) -> Bound<'p, PyArray2<bool>> {
        PyArray2::from_array_bound(py, &self.inner.exhaustive)
    }
    fn n_samples<'p>(&self, py: Python<'p>) -> Bound<'p, PyArray2<u64>> {
        PyArray2::from_array_bound(py, &self.inner.cnt.sum_axis(pd::INPUT_AXIS))
    }
}

impl PyProbeDistribution {
    fn from_inner(inner: pd::ProbeDistribution<String>) -> Self {
        std::sync::RwLock::new(inner).into()
    }
    fn read(&self) -> std::sync::RwLockReadGuard<'_, pd::ProbeDistribution<String>> {
        self.inner.read().unwrap()
    }
    fn write(&self) -> std::sync::RwLockWriteGuard<'_, pd::ProbeDistribution<String>> {
        self.inner.write().unwrap()
    }
}

#[pymethods]
impl PyProbeDistribution {
    #[new]
    fn new(wires: Vec<String>) -> Self {
        Self::from_inner(pd::ProbeDistribution::from_wires(wires))
    }
    fn leak_wire<'p>(&mut self, py: Python<'p>, var: String, p: f64) -> Self {
        py.allow_threads(|| Self::from_inner(self.write().leak_wire(var, p)))
    }
    fn bin_op<'p>(
        &mut self,
        py: Python<'p>,
        dest: String,
        src1: String,
        src2: String,
        p: f64,
    ) -> Self {
        py.allow_threads(|| Self::from_inner(self.write().bin_op(dest, src1, src2, p)))
    }
    fn split_wire<'p>(
        &mut self,
        py: Python<'p>,
        src: String,
        dest1: String,
        dest2: String,
    ) -> Self {
        py.allow_threads(|| Self::from_inner(self.write().split_wire(src, dest1, dest2)))
    }
    fn apply_op<'p>(
        &mut self,
        py: Python<'p>,
        inputs: Vec<String>,
        outputs: Vec<String>,
        pdt: &PyPDT,
    ) -> Self {
        py.allow_threads(|| Self::from_inner(self.write().apply_op(inputs, outputs, &pdt.inner)))
    }
    fn to_vec(&self) -> Vec<f64> {
        self.read().get_distr().to_vec()
    }
    fn wires(&self) -> Vec<String> {
        self.read().wires.clone()
    }
    fn distr(&self) -> Vec<f64> {
        self.read().distr.as_slice().unwrap().to_owned()
    }
    fn wire_idx(&self, wire: String) -> u32 {
        self.read().wire_idx(&wire)
    }
}

#[pymethods]
impl PyPDT {
    fn to_array<'p>(&self, py: Python<'p>) -> Bound<'p, PyArray2<f64>> {
        self.inner.to_pyarray_bound(py)
    }
}

#[pymethods]
impl PyGPdt {
    fn instantiate<'p>(&self, py: Python<'p>, p: f64) -> PyPDT {
        py.allow_threads(|| self.inner.instantiate(p)).into()
    }
    fn to_array<'p>(&self, py: Python<'p>) -> Bound<'p, PyArray3<f64>> {
        self.inner.as_ratios().to_pyarray_bound(py)
    }
}

fn vecs2graph(
    vars: Vec<(u32, Vec<usize>, String)>,
    in_ports: Vec<Option<(usize, usize)>>,
    out_ports: Vec<Option<(usize, usize)>>,
) -> Result<Vec<circuit::Var>, SErr> {
    vars.into_iter()
        .zip(in_ports)
        .zip(out_ports)
        .map(|(((kind, mut ops, name), in_port), out_port)| {
            ops.sort_unstable();
            Ok(circuit::Var {
                src: match (kind, in_port) {
                    (0, Some((p, s))) => circuit::VarSrc::Input(p, s),
                    (0, None) => {
                        return Err(SErr("Input has no port".to_owned()));
                    }
                    (1, None) => circuit::VarSrc::Random,
                    (2, None) => {
                        circuit::VarSrc::Sum(ops.try_into().map_err(|_| "Wrong ops count")?)
                    }
                    (3, None) => {
                        circuit::VarSrc::Product(ops.try_into().map_err(|_| "Wrong ops count")?)
                    }
                    (4, None) => {
                        circuit::VarSrc::Not(ops.try_into().map_err(|_| "Wrong ops count")?)
                    }
                    (_, p) => {
                        return Err(SErr(format!("Invalid op kind {} or port {:?}", kind, p)));
                    }
                },
                output_port: out_port,
                name: name,
            })
        })
        .collect()
}
