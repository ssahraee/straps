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
    default_n_s_max = 10 ** 6
    default_suff_thresh = 1000
    max_d = 5
else:
    # Quite tight (?), but a bit heavy
    default_n_s_max = 10 ** 7
    default_suff_thresh = 1000
    max_d = 5

default_err = 1e-6
# default_p = np.logspace(-6, 0, 50)
default_p = np.logspace(-4, 0, 40)
default_ds = list(range(1, max_d + 1))

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


def data_fig(
    fn,
    ds=None,
    err=default_err,
    p=default_p,
    n_s_max=default_n_s_max,
    suff_thresh=default_suff_thresh,
    use_copy=True,
    cum_tr=False,
):
    if isinstance(ds, int):
        ds = range(1, ds + 1)
    if ds is None:
        ds = default_ds
    ds = list(ds)

    def eval_all(k):
        return [
            eval_circs.eval_circ_all_p(
                fn, k, err, d, p, n_s_max, suff_thresh, use_copy, cum_tr
            )
            for d in ds
        ]

    ubs = eval_all("ub")
    lbs = eval_all("lb")
    return {
        "fn": fn,
        "err": err,
        "ds": ds,
        "p": p,
        "n_s_max": n_s_max,
        "suff_thresh": suff_thresh,
        "ubs": ubs,
        "lbs": lbs,
        "use_copy": use_copy,
        "cum_tr": cum_tr,
    }


def main():
    import argparse

    supported_circuits = ", ".join(eval_circs.specialized_circuits.keys())
    parser = argparse.ArgumentParser(description="Plot a simple STRAPS result.")
    parser.add_argument(
        "circuit",
        type=str,
        help="The circuit to analyze. Possible options: " + supported_circuits,
    )
    parser.add_argument(
        "-d",
        "--nshares",
        type=str,
        default="1,2,3,4",
        help="Number of shares to consider. Default: 1,2,3,4.",
    )
    parser.add_argument(
        "--nmax",
        type=int,
        default=10 ** 5,
        help="N_max parameter (see the paper). Default: 10^5.",
    )
    parser.add_argument(
        "--nt",
        type=int,
        default=10 ** 3,
        help="N_t parameter (see the paper). Default: 1000.",
    )
    parser.add_argument(
        "--err",
        type=float,
        default=1e-6,
        help="Confidence level of the bounds. Default: 1e-6.",
    )
    parser.add_argument(
        "--pmin",
        type=float,
        default=1e-4,
        help="Minimum value for the p parameter of the random probing model. Default: 1e-4.",
    )
    parser.add_argument(
        "--pmax",
        type=float,
        default=1e0,
        help="Maximum value for the p parameter of the random probing model. Default: 1.",
    )
    parser.add_argument(
        "--np",
        type=int,
        default=40,
        help="Number of (log-spaced) values for the p parameter. Default: 40.",
    )

    args = parser.parse_args()

    fn = args.circuit
    if fn not in eval_circs.specialized_circuits:
        print("Error: circuit {} not supported".format(fn))
        print("Supported circuits:", supported_circuits)
        return
    ds = eval("(" + args.nshares + ")")
    if isinstance(ds, int):
        ds = (ds,)
    err = args.err
    p = np.logspace(np.log10(args.pmin), np.log10(args.pmax), args.np)
    n_s_max = args.nmax
    suff_thresh = args.nt

    plot_fig(**data_fig(fn, ds, err, p, n_s_max, suff_thresh))
    plt.show()


if __name__ == "__main__":
    main()
