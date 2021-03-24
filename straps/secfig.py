# STRAPS - Statistical Testing of RAndom Probing Security
# Copyright (C) 2021 UCLouvain
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.


"""Top-level functions to get the data for a plot and then plot it."""

import numpy as np
from matplotlib import pyplot as plt

from . import eval_circs

light = False
if light:
    # Reasonnably light, but not very tight for d=6 or for large p
    default_n_s_max = 10**6
    default_suff_thresh = 1000
    max_d = 5
else:
    # Quite tight (?), but a bit heavy
    default_n_s_max = 10**7
    default_suff_thresh = 1000
    max_d = 5

default_err = 1e-6
#default_p = np.logspace(-6, 0, 50)
default_p = np.logspace(-4, 0, 40)
default_ds = list(range(1, max_d+1))

plt_fn = plt.loglog

def plot_fig(fn, err, ds, p, ubs, lbs, n_s_max, suff_thresh, use_copy, cum_tr):
    prop_cycle = plt.rcParams["axes.prop_cycle"]
    colors = prop_cycle.by_key()["color"]
    for i, d in enumerate(ds):
        plt_fn(p, np.minimum(1, lbs[i]), linestyle="--", color=colors[i])
        plt_fn(p, np.minimum(1, ubs[i]), "-", label="d = {}".format(d), color=colors[i])
    plt.legend()
    plt.xlabel("p")
    plt.ylabel("security")
    plt_title = "{} err={:.0e} nt={:.0e} ns={:.0e}".format(
        eval_circs.specialized_circuits[fn][0], err, n_s_max, suff_thresh
        )
    if use_copy:
        plt_title += " uc"
    if cum_tr:
        plt_title += " ct"
    plt.title(plt_title)

def data_fig(fn, ds=None, err=default_err, p=default_p,
        n_s_max=default_n_s_max, suff_thresh=default_suff_thresh,
        use_copy=True, cum_tr=False):
    if isinstance(ds, int):
        ds = range(1, ds+1)
    if ds is None:
        ds = default_ds
    ds = list(ds)
    def eval_all(k):
        return [eval_circs.eval_circ_all_p(fn, k, err, d, p, n_s_max,
            suff_thresh, use_copy, cum_tr) for d in ds]
    ubs = eval_all('ub')
    lbs = eval_all('lb')
    return {
        'fn': fn,
        'err': err,
        'ds': ds,
        'p': p,
        'n_s_max': n_s_max,
        'suff_thresh': suff_thresh,
        'ubs': ubs,
        'lbs': lbs,
        'use_copy': use_copy,
        'cum_tr': cum_tr,
        }
