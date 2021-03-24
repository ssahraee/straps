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

use super::combinatorics::{count_selections, MWCombinations};
use indicatif::ProgressStyle;
use itertools::Itertools;
use ndarray::s;
use ordered_float::OrderedFloat;
use rand::Rng;
use rayon::prelude::*;
use std::ops::AddAssign;

pub const NPROBES_AXIS: ndarray::Axis = ndarray::Axis(0);
pub const INPUT_AXIS: ndarray::Axis = ndarray::Axis(1);
pub const OUTPUT_AXIS: ndarray::Axis = ndarray::Axis(2);

pub(crate) struct CntSim<Gadget: super::gadget::Gadget> {
    gadget: Gadget,
    // maximum number of adversarial probes (sum of all var use counts)
    max_nb_probes: u32,
    n_inputs: u32,
    n_outputs: u32,
    pp_sel_map: Vec<usize>,
    // minimum number of vars the can be touched by a gien number of probes
    n_probes_n_min_pp: Vec<usize>,
}
impl<Gadget: super::gadget::Gadget + Sync> CntSim<Gadget> {
    pub fn new(gadget: Gadget) -> Self {
        let max_nb_probes = gadget.max_n_probes();
        let n_inputs = gadget.n_inputs() as u32;
        let n_outputs = gadget.n_outputs() as u32;
        let n_probes_n_min_pp = std::iter::once(0)
            .chain(
                gadget
                    .pp_maxp()
                    .iter()
                    .copied()
                    .enumerate()
                    .flat_map(|(i, use_count)| std::iter::repeat(i + 1).take(use_count as usize)),
            )
            .collect::<Vec<_>>();
        assert!(n_probes_n_min_pp[max_nb_probes as usize] == gadget.n_pp());
        let mut ppu = gadget.pp_maxp().to_vec();
        ppu.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(ppu.as_slice(), gadget.pp_maxp());
        let pp_sel_map = gadget
            .pp_maxp()
            .iter()
            .enumerate()
            .flat_map(|(i, x)| std::iter::repeat(i).take(*x as usize))
            .collect::<Vec<_>>();
        return Self {
            gadget,
            max_nb_probes,
            n_inputs,
            n_outputs,
            pp_sel_map,
            n_probes_n_min_pp,
        };
    }
    fn n_used_vars(&self) -> usize {
        self.gadget.n_pp()
    }
    fn n_nprobe_cases(&self) -> usize {
        self.max_nb_probes as usize + 1
    }
    fn inputs2id(&self, inputs: impl Iterator<Item = usize>) -> usize {
        inputs.map(|i| 1 << i).sum()
    }
    fn iter_probe_set_min_weight(
        &self,
        k: usize,
        min_weight: u32,
    ) -> impl Iterator<Item = Vec<usize>> + '_ {
        MWCombinations::new(self.gadget.pp_maxp(), k, min_weight)
    }

    fn probe_set_pmask(&self, output_id: usize, probe_sel: &[bool]) -> usize {
        self.probe_set_pp(
            output_id,
            probe_sel
                .iter()
                .positions(|x| *x)
                .map(|i| self.pp_sel_map[i]),
        )
    }
    fn probe_set_pp(&self, output_id: usize, pp: impl Iterator<Item = usize> + Clone) -> usize {
        self.inputs2id(
            self.gadget
                .sim_probes(super::utils::iter_set_bits(output_id), pp)
                .into_iter(),
        )
    }

    fn gen_sel(&self, n_probes: usize) -> Vec<bool> {
        let n = self.pp_sel_map.len();
        if n_probes > n / 2 {
            let mut res = self.gen_sel(n - n_probes);
            for x in res.iter_mut() {
                *x = !*x;
            }
            return res;
        } else {
            let mut res = vec![false; n];
            let mut rng = rand::thread_rng();
            // Robert Floyd's algorithm
            for j in (n - n_probes as usize)..n {
                let r = rng.gen_range(0, j + 1);
                if res[r] {
                    res[j] = true;
                } else {
                    res[r] = true;
                }
            }
            return res;
        }
    }

    fn probe_all_nprobes(
        &self,
        output_id: usize,
        n_probes: std::ops::Range<usize>,
        progress: &super::multiprogress::SubProgress,
    ) -> ndarray::Array2<u64> {
        firestorm::profile_fn!(probe_all_nprobes);
        let mut res = ndarray::Array2::zeros((1 << self.n_inputs, n_probes.len()));
        if n_probes.end != 0 && n_probes.start <= self.max_nb_probes as usize {
            let n_pp_min = self.n_probes_n_min_pp[n_probes.start];
            let n_pp_max = std::cmp::min(n_probes.end - 1, self.n_used_vars());
            for n_pp in n_pp_min..=n_pp_max {
                let pp_sets = self
                    .iter_probe_set_min_weight(n_pp, n_probes.start as u32)
                    .collect::<Vec<_>>();
                let tmp_res = pp_sets
                    .into_par_iter()
                    .fold(
                        || ndarray::Array2::zeros((1 << self.n_inputs, n_probes.len())),
                        |mut sub_res, pp_set| {
                            let input_offset = self.probe_set_pp(output_id, pp_set.iter().copied());
                            let use_counts = pp_set
                                .iter()
                                .map(|probe| self.gadget.pp_maxp()[*probe] as usize)
                                .collect::<Vec<_>>();
                            let nb_sels = count_selections(n_probes.clone(), &use_counts);
                            sub_res.slice_mut(s![input_offset, ..]).add_assign(&nb_sels);
                            progress.inc(1);
                            sub_res
                        },
                    )
                    .reduce_with(|sub_res1, sub_res2| sub_res1 + sub_res2);
                if let Some(x) = tmp_res {
                    res.add_assign(&x);
                }
            }
            return res;
            //let pp_sets = (n_pp_min..=n_pp_max).flat_map(|n_pp| self.iter_probe_set_min_weight(n_pp, n_probes.start as u32));
            /*
            return (n_pp_min..=n_pp_max)
                .into_par_iter()
                .flat_map(|n_pp| {
                    rayon::iter::split(
                        super::combinatorics3::MWCombinations::new(
                            self.gadget.pp_maxp(),
                            n_pp,
                            n_probes.start as u32,
                        ),
                        super::combinatorics3::MWCombinations::split,
                    )
                    .flat_map_iter(|x| x)
                })
                .collect::<Vec<_>>()
                .into_par_iter()
                .fold(
                    || res.clone(),
                    |mut sub_res, pp_set| {
                        let input_offset = self.probe_set_pp(output_id, pp_set.iter().copied());
                        let use_counts = pp_set
                            .iter()
                            .map(|probe| self.gadget.pp_maxp()[*probe] as usize)
                            .collect::<Vec<_>>();
                        let nb_sels = count_selections(n_probes.clone(), &use_counts);
                        sub_res.slice_mut(s![input_offset, ..]).add_assign(&nb_sels);
                        progress.inc(1);
                        sub_res
                    },
                )
                .reduce(|| res.clone(), |sub_res1, sub_res2| sub_res1 + sub_res2);
                */
        } else {
            return res;
        }
    }

    fn costs_n_probes_grow(&self) -> impl Iterator<Item = (usize, f64)> + '_ {
        (0..self.n_nprobe_cases()).map(move |n_probes| {
            (
                n_probes,
                (0..=std::cmp::min(n_probes, self.n_used_vars()))
                    .map(|n_vars| {
                        statrs::function::factorial::binomial(
                            self.n_used_vars() as u64,
                            n_vars as u64,
                        )
                    })
                    .sum(),
            )
        })
    }
    fn costs_n_probes_shrink(&self) -> impl Iterator<Item = (usize, f64)> + '_ {
        (0..self.n_nprobe_cases()).rev().map(move |n_probes| {
            let n_sets = (self.n_probes_n_min_pp[n_probes]..=self.n_used_vars())
                .map(|n_pp| {
                    self.iter_probe_set_min_weight(n_pp, n_probes as u32)
                        .count() as f64
                })
                .sum();
            (n_probes, n_sets)
        })
    }
    fn make_incr<'a>(
        src: impl Iterator<Item = (usize, f64)>,
    ) -> impl Iterator<Item = (usize, f64)> {
        let mut src = src.peekable();
        src.peek().cloned().into_iter().chain(
            src.tuple_windows()
                .map(|((_, a), (n_probes, b))| (n_probes, b - a)),
        )
    }

    fn probe_samples(
        &self,
        n_s_max: u32,
        suff_thresh: u32,
        n_sets: f64,
        output_id: usize,
        n_probes: usize,
    ) -> (Vec<u64>, u64) {
        firestorm::profile_fn!(probe_samples);
        let mut res = vec![0u64; 1 << self.n_inputs];
        let mut n_probings = 0u64;
        let mut range = 0..suff_thresh;
        let end: u32 = std::cmp::min(n_sets as u32, n_s_max);
        let add_vecs = |mut r1: Vec<u64>, r2: Vec<u64>| {
            r1.iter_mut().zip(r2.iter()).for_each(|(x, y)| *x += *y);
            r1
        };
        while !range.is_empty() {
            let tmp_res = range
                .clone()
                .into_par_iter()
                .fold_with(vec![0u64; 1 << self.n_inputs], |mut tmp_res, _| {
                    let probe_sel = self.gen_sel(n_probes);
                    let input_offset = self.probe_set_pmask(output_id, &probe_sel);
                    tmp_res[input_offset] += 1;
                    tmp_res
                })
                .reduce(|| vec![0u64; 1 << self.n_inputs], add_vecs);
            res = add_vecs(res, tmp_res);
            n_probings += (range.end - range.start) as u64;
            if res[res.len() - 1] >= (suff_thresh as u64) {
                break;
            }
            range = range.end..std::cmp::min(2 * range.end, end);
        }
        return (res, n_probings);
    }

    /// Return (Some(x), _) only when reaching the n_s_max upper bound.
    fn probe_auto_samples_inner(
        &self,
        n_s_max: u32,
        suff_thresh: u32,
        n_sets: f64,
        output_id: usize,
        n_probes: usize,
    ) -> (Option<SampleRes>, u64) {
        firestorm::profile_fn!(probe_auto_samples_inner);
        let (counts, cost1) = self.probe_samples(n_s_max, suff_thresh, n_sets, output_id, n_probes);
        let tot_samples = counts.iter().copied().sum::<u64>() as u32;
        let (counts, cost2) = if tot_samples as f64 == n_sets {
            (None, 0)
        } else if tot_samples == n_s_max {
            (Some(counts), 0)
        } else {
            let (counts, cost2) =
                self.probe_samples(n_s_max, suff_thresh, n_sets, output_id, n_probes);
            (Some(counts), cost2)
        };
        let res = counts.map(|counts| SampleRes {
            n_probes,
            output_index: output_id,
            counts,
            exhaustive: false,
        });
        return (res, cost1 + cost2);
    }

    pub(crate) fn probe_output<'a>(
        &'a self,
        output_id: usize,
        n_s_max: u32,
        suff_thresh: u32,
        progress: &'a super::multiprogress::SubProgress,
    ) -> impl Iterator<Item = SampleRes> + 'a {
        firestorm::profile_fn!(probe_output);
        //) -> impl ParallelIterator<Item = SampleRes> + 'a {
        let mut inc_costs_grow = Self::make_incr(self.costs_n_probes_grow()).peekable();
        let mut inc_costs_shrink = Self::make_incr(self.costs_n_probes_shrink()).peekable();
        let acc_cost_nprobes_low =
            |(_, cost_acc), (n_probes, cost_inc)| (n_probes + 1, cost_acc + cost_inc);
        let acc_cost_nprobes_high =
            |(_, cost_acc), (n_probes, cost_inc)| (n_probes, cost_acc + cost_inc);
        // [0, exh_low_ub) interval will be done by exhaustion
        let (exh_low_ub, cost_low) = inc_costs_grow
            .peeking_take_while(|(n_probes, cost)| {
                *n_probes <= self.max_nb_probes as usize && *cost <= suff_thresh as f64
            })
            .fold((0, 0.0), acc_cost_nprobes_low);
        let (exhaust_high_min_n_probes, cost_high) = inc_costs_shrink
            .peeking_take_while(|(n_probes, cost)| {
                *n_probes >= exh_low_ub && *cost <= suff_thresh as f64
            })
            .fold((self.n_nprobe_cases(), 0.0), acc_cost_nprobes_high);
        let approx_cost = cost_low as i64
            + cost_high as i64
            + (exhaust_high_min_n_probes - exh_low_ub) as i64 * n_s_max as i64;
        progress.inc_length(approx_cost);
        // Test if exhaustive sampling is cheaper than random sampling.
        let try_random_sampling = |n_probes, cost_exhaust| {
            let (counts, cost) = self.probe_auto_samples_inner(
                n_s_max,
                suff_thresh,
                cost_exhaust,
                output_id,
                n_probes,
            );
            progress.inc_length(cost as i64);
            progress.inc(cost as i64);
            return counts.is_none();
        };
        let (mut exh_low_ub, mut cost_low) = {
            firestorm::profile_section!(try_g_low);
            inc_costs_grow
                .take_while(|(n_probes, cost_exhaust)| {
                    *n_probes < exhaust_high_min_n_probes
                        && try_random_sampling(*n_probes, *cost_exhaust)
                })
                .fold((exh_low_ub, cost_low), acc_cost_nprobes_low)
        };
        let (mut exhaust_high_min_n_probes, mut cost_high) = {
            firestorm::profile_section!(try_g_high);
            inc_costs_shrink
                .take_while(|(n_probes, cost_exhaust)| {
                    *n_probes >= exh_low_ub && try_random_sampling(*n_probes, *cost_exhaust)
                })
                .fold(
                    (exhaust_high_min_n_probes, cost_high),
                    acc_cost_nprobes_high,
                )
        };
        assert!(exh_low_ub <= exhaust_high_min_n_probes);
        if exh_low_ub == exhaust_high_min_n_probes {
            // We can do a single probe all !
            // Make part2 and part3 empty
            exh_low_ub = self.n_nprobe_cases();
            exhaust_high_min_n_probes = exh_low_ub;
            cost_low = (1 << self.n_used_vars()) as f64;
            cost_high = 0.0;
        }
        let l = progress.length() as i64;
        progress.inc_length(progress.position() as i64);
        progress.inc_length(-l);
        assert_eq!(progress.length(), progress.position());
        progress.inc_length(cost_low as i64);
        progress.inc_length(cost_high as i64);
        progress.inc_length(n_s_max as i64 * (exhaust_high_min_n_probes - exh_low_ub) as i64);
        progress.finishing(true);
        // There are three regions:
        // 1. exhaustive sampling for [0, exh_low_ub)
        // 2. random sampling for [ exh_low_ub, exhaust_high_min_n_probes)
        // 3. exhaustive sampling for [exhaust_high_min_n_probes, self.max_nb_probes+1)
        let make_sample_res_exh =
            |(n_probes, counts): (usize, ndarray::ArrayView1<u64>)| SampleRes {
                n_probes,
                output_index: output_id,
                counts: counts.to_vec(),
                exhaustive: true,
            };
        // part 1.
        let res1_vec = {
            firestorm::profile_section!(res1);
            self.probe_all_nprobes(output_id, 0..exh_low_ub, progress)
                .axis_iter(ndarray::Axis(1))
                .enumerate()
                .map(make_sample_res_exh)
                .collect::<Vec<_>>()
        };
        // part 3.
        let res3_vec = {
            firestorm::profile_section!(res3);
            self.probe_all_nprobes(
                output_id,
                exhaust_high_min_n_probes..self.n_nprobe_cases(),
                progress,
            )
            .axis_iter(ndarray::Axis(1))
            .enumerate()
            .map(move |(i, counts)| make_sample_res_exh((exhaust_high_min_n_probes + i, counts)))
            .collect::<Vec<_>>()
        };
        // part 2.
        let res2 = {
            firestorm::profile_section!(res2);
            (exh_low_ub..exhaust_high_min_n_probes)
                //.into_par_iter()
                .map(move |n_probes| {
                    let (counts, cost) = self.probe_auto_samples_inner(
                        n_s_max,
                        suff_thresh,
                        (n_s_max + 1) as f64,
                        output_id,
                        n_probes,
                    );
                    progress.inc_length(cost as i64 - n_s_max as i64);
                    progress.inc(cost as i64);
                    counts.unwrap()
                })
                .collect::<Vec<_>>()
        };
        res1_vec
            .into_iter()
            .chain(res3_vec.into_iter())
            .chain(res2.into_iter())
    }

    fn collect_pdtcols(&self, pdtcols: impl IntoIterator<Item = SampleRes>) -> CntSimSt {
        firestorm::profile_fn!(collect_pdtcols);
        let n_nprobes_cases = self.n_nprobe_cases();
        let n_input_cases = 1 << self.n_inputs;
        let n_output_cases = 1 << self.n_outputs;
        let mut res = CntSimSt::new(n_nprobes_cases, n_input_cases, n_output_cases);
        let mut init = ndarray::Array2::from_elem((n_output_cases, n_nprobes_cases), false);
        for pdtcol in pdtcols {
            assert_eq!(pdtcol.counts.len(), n_input_cases);
            assert!(!init[(pdtcol.output_index, pdtcol.n_probes)]);
            for (i, cnt) in pdtcol.counts.iter().enumerate() {
                res.cnt[(pdtcol.n_probes, i, pdtcol.output_index)] = *cnt;
            }
            res.exhaustive[(pdtcol.n_probes, pdtcol.output_index)] = pdtcol.exhaustive;
            init[(pdtcol.output_index, pdtcol.n_probes)] = true;
        }
        assert!(init.iter().all(|x| *x));
        return res;
    }

    pub fn run_sampling(&self, n_s_max: u32, suff_thresh: u32) -> CntSimSt {
        firestorm::profile_fn!(run_sampling);
        assert!(n_s_max >= suff_thresh);
        let n_output_cases: usize = 1 << self.n_outputs;
        let style = ProgressStyle::default_bar()
            .template("{msg} [{bar:40}] {pos}/{len} [{elapsed_precise}>{eta_precise}]");
        let progress = super::multiprogress::MultiProgressConfig::new(n_output_cases, style);
        progress.run(|mp| {
            self.collect_pdtcols(
                (0..n_output_cases)
                    //.into_par_iter()
                    .flat_map(|output_id| {
                        let local_progress = mp.sub(output_id);
                        self.probe_output(output_id, n_s_max, suff_thresh, local_progress)
                    })
                    .collect::<Vec<_>>()
                    .into_iter(),
            )
        })
    }
}

#[derive(Debug)]
pub(crate) struct SampleRes {
    pub(crate) n_probes: usize,
    pub(crate) output_index: usize,
    pub(crate) counts: Vec<u64>,
    pub(crate) exhaustive: bool,
}

#[derive(Debug, Clone)]
pub struct CntSimSt {
    pub cnt: ndarray::Array3<u64>,
    pub exhaustive: ndarray::Array2<bool>,
}
impl CntSimSt {
    fn new(n_nprobes_cases: usize, n_input_cases: usize, n_output_cases: usize) -> Self {
        Self {
            cnt: ndarray::Array3::zeros((n_nprobes_cases, n_input_cases, n_output_cases)),
            exhaustive: ndarray::Array2::from_elem((n_nprobes_cases, n_output_cases), false),
        }
    }
    fn n_nprobes_cases(&self) -> usize {
        self.cnt.shape()[NPROBES_AXIS.index()]
    }
    fn n_input_cases(&self) -> usize {
        self.cnt.shape()[INPUT_AXIS.index()]
    }
    fn n_output_cases(&self) -> usize {
        self.cnt.shape()[OUTPUT_AXIS.index()]
    }
    pub fn estimate(&self) -> GPdt {
        let tot = self.cnt.sum_axis(INPUT_AXIS);
        let ratios = ndarray::Array::from_shape_fn(self.cnt.raw_dim(), |(i, j, k)| {
            (self.cnt[(i, j, k)] as f64) / (tot[(i, k)] as f64)
        });
        return GPdt { ratios };
    }
    fn bound_margin(&self, err: f64) -> f64 {
        err / ((self.n_input_cases() * self.n_output_cases() * self.n_nprobes_cases()) as f64)
    }
    fn bound(&self, err: f64, ub: bool, cum_tr: bool) -> GPdt {
        let mut bound = ndarray::Array3::<f64>::zeros(self.cnt.raw_dim());
        let tot = self.cnt.sum_axis(INPUT_AXIS);
        let n = self.n_input_cases();
        let margin = self.bound_margin(err);
        // Use a cache, as we often have many identical n values, hence k repeats significantly
        let new_cache = |n| {
            let mut cache = std::collections::BTreeMap::new();
            move |k| {
                *cache.entry(k).or_insert_with(|| {
                    if ub {
                        super::utils::binom_param_ub(n, k, margin)
                    } else {
                        super::utils::binom_param_lb(n, k, margin)
                    }
                })
            }
        };
        bound
            .axis_iter_mut(ndarray::Axis(0))
            .into_par_iter()
            .enumerate()
            .for_each(|(k, mut bound)| {
                bound
                    .axis_iter_mut(ndarray::Axis(1))
                    .into_par_iter()
                    .enumerate()
                    .for_each(|(j, mut bound)| {
                        if self.exhaustive[(k, j)] {
                            for i in 0..n {
                                bound[i] = (self.cnt[(k, i, j)] as f64) / (tot[(k, j)] as f64);
                            }
                        } else {
                            let mut counts = self.cnt.slice(s![k, .., j]).to_vec();
                            let tot = counts.iter().copied().sum::<u64>();
                            if cum_tr {
                                super::cum_transform::cum_transform(&mut counts);
                            }
                            //let tot = counts[0];
                            let mut cached_binom_param_bound = new_cache(tot);
                            let mut cum_bounds = counts
                                .iter()
                                .map(|x| OrderedFloat(cached_binom_param_bound(*x)))
                                .collect::<Vec<_>>();
                            if cum_tr {
                                if ub {
                                    super::cum_transform::cum_transform_inv_positive(
                                        &mut cum_bounds,
                                    );
                                } else {
                                    super::cum_transform::cum_transform_inv_min_positive(
                                        &mut cum_bounds,
                                    );
                                }
                            } else {
                                cum_bounds[0] = OrderedFloat(
                                    0.0f64.max(
                                        1.0 - cum_bounds[1..]
                                            .iter()
                                            .map(|x| x.into_inner())
                                            .sum::<f64>(),
                                    ),
                                );
                            }
                            bound
                                .slice_mut(s![..])
                                .iter_mut()
                                .zip(cum_bounds.iter())
                                .for_each(|(b, c)| *b = c.into_inner());
                        }
                    });
            });
        return GPdt { ratios: bound };
    }
    pub fn ub(&self, err: f64, cum_tr: bool) -> GPdt {
        self.bound(err, true, cum_tr)
    }
    pub fn lb(&self, err: f64, cum_tr: bool) -> GPdt {
        self.bound(err, false, cum_tr)
    }
}

pub struct GPdt {
    ratios: ndarray::Array3<f64>,
}
impl GPdt {
    pub fn instantiate(&self, p: f64) -> ndarray::Array2<f64> {
        let n_max_probes = (self.ratios.shape()[0] - 1) as i32;
        let coefs = (0..=n_max_probes)
            .map(|i| {
                p.powi(i)
                    * (1.0 - p).powi(n_max_probes - i)
                    // use statrs instead of num_integer as it gives directly a f64
                    * statrs::function::factorial::binomial(n_max_probes as u64, i as u64)
            })
            .collect::<Vec<f64>>();
        return ndarray::Array::from_shape_fn(
            (self.ratios.shape()[1], self.ratios.shape()[2]),
            |(i, j)| {
                coefs
                    .iter()
                    .enumerate()
                    .map(|(k, c)| self.ratios[(k, i, j)] * c)
                    .sum()
            },
        );
    }
    pub fn as_ratios(&self) -> &ndarray::Array3<f64> {
        &self.ratios
    }
}
