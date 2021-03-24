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


"""Build composite shared circuits and evaluate their security."""

import numpy as np
import tqdm

from .sh_pdt import ShPd
from . import pdt_sampling
from . import utils

## Generic circuit builders


def eval_aes_sbox(p, pdts, ref=True, d=2, ref_name="optref"):
    """AES S-box in GF(256). See paper for structure."""
    # Create the Shared PD with one output sharing
    x = ShPd(["out"], d)
    # We build the circuit from the output: we start from the output sharing,
    # create the gadget that generates it, then work backwards until we reach
    # the intput.

    # Apply the output multiplication of the AES S-box
    # out = m3i1 * m3i2 (with ISW multiplication)
    x.op("out", ["m3i1", "m3i2"], pdts["ISW"])
    # Another multiplication
    # m3i1 = m2i1 * m2i2
    x.op("m3i1", ["m2i1", "m2i2"], pdts["ISW"])
    # Let m2i2 <- m2i2**(2**4) (we can override variables)
    for _ in range(4):
        x.op("m2i2", ["m2i2"], pdts["square"])
    # m2i2 = m1i1 * m1i2
    x.op("m2i2", ["m1i1", "m1i2"], pdts["ISW"])
    # Copy gate: m2i1 <- t0 and m1i1 <- t0.
    x.split_sharing("t0", "m2i1", "m1i1")
    # Refresh: t0 <- Refresh(t0)
    if ref:
        x.op("t0", ["t0"], pdts[ref_name])
    # t0 <- t0**(2**2)
    for _ in range(2):
        x.op("t0", ["t0"], pdts["square"])
    # Copy gate: t0 <- m0o and m1i2 <- m0o
    x.split_sharing("m0o", "t0", "m1i2")
    # m0o = m0i1 * m0i2
    x.op("m0o", ["m0i1", "m0i2"], pdts["ISW"])
    # Copy gate: m0i2 <- t1 and m3i2 <- t1
    x.split_sharing("t1", "m0i2", "m3i2")
    # Refresh: t1 <- Refresh(t1)
    if ref:
        x.op("t1", ["t1"], pdts[ref_name])
    # t1 <- t1**2
    x.op("t1", ["t1"], pdts["square"])
    # Copy gate: m0i1 <- in and t1 <- in
    x.split_sharing("in", "m0i1", "t1")
    return x.security("in")


def eval_x_cube(p, pdts, ref=True, d=2, ref_name="optref"):
    x = ShPd(["out"], d)
    x.op("out", ["t0", "t1"], pdts["ISW"])
    if ref:
        x.op("t0", ["t0"], pdts[ref_name])
    x.op("t0", ["t0"], pdts["square"])
    x.split_sharing("in", "t0", "t1")
    return x.security("in")


def eval_mul_both(p, pdts, d=2):
    """Probability that either of the two full input sharings are required for simulation."""
    x = ShPd(["out"], d)
    x.op("out", ["t0", "t1"], pdts["ISW"])
    return sum(x.distr()[i * 2 ** d + 2 ** d - 1] for i in range(2 ** d)) + np.sum(
        x.distr()[-(2 ** d) : -1]
    )


def eval_n_sharings(p, pdts, d=2, circ_name=None, n_inputs=1, sec_input=0):
    x = ShPd(["out"], d)
    x.op("out", ["t{}".format(i) for i in range(n_inputs)], pdts[circ_name])
    return x.security("t{}".format(sec_input))


# Evaluation function and list of the gadgets that compose the circuit
base_circuits = {
    "eval_aes_sbox": (
        eval_aes_sbox,
        lambda **kwargs: ["ISW", "simpleref", "optref", "square"],
    ),
    "eval_x_cube": (
        eval_x_cube,
        lambda **kwargs: ["ISW", "simpleref", "optref", "square"],
    ),
    "eval_mul_both": (eval_mul_both, lambda **kwargs: ["ISW"]),
    "eval_n_sharings": (eval_n_sharings, lambda **kwargs: [kwargs["circ_name"]]),
}


## Circuits that are specialization of the base generic circuits
# name: (full_name, base_circuit_name, base_circuit_kwargs)
specialized_circuits = {
    "aes_sbox": ("AES S-box", "eval_aes_sbox", {"ref_name": "optref"}),
    "aes_sbox_noref": ("AES S-box no refresh", "eval_aes_sbox", {"ref": False}),
    "aes_sbox_simpleref": (
        "AES S-box simple refresh",
        "eval_aes_sbox",
        {"ref_name": "simpleref"},
    ),
    "cube_isw_noref": ("ISW(x^2, x)", "eval_x_cube", {"ref": False}),
    "cube_isw_optref": ("ISW(SNI-ref(x^2), x)", "eval_x_cube", {"ref_name": "optref"}),
    "cube_isw_simpleref": (
        "ISW(simple-ref(x^2), x)",
        "eval_x_cube",
        {"ref_name": "simpleref"},
    ),
    "int_sq_isw_optref": (
        "single circuit ISW(x, opt-ref(x))",
        "eval_n_sharings",
        {"circ_name": "isw_sq_opt"},
    ),
    "int_sq_isw_simpleref": (
        "single circuit ISW(x, simple-ref(x))",
        "eval_n_sharings",
        {"circ_name": "isw_sq_simple"},
    ),
    "int_sq_isw_noref": (
        "single circuit ISW(x, x)",
        "eval_n_sharings",
        {"circ_name": "isw_sq_noref"},
    ),
    "int_cube_isw_optref": (
        "single circuit ISW(x, opt-ref(x^2))",
        "eval_n_sharings",
        {"circ_name": "isw_cube_opt"},
    ),
    "int_cube_isw_simpleref": (
        "single circuit ISW(x, simple-ref(x^2))",
        "eval_n_sharings",
        {"circ_name": "isw_cube_simple"},
    ),
    "int_cube_isw_noref": (
        "single circuit ISW(x, x^2)",
        "eval_n_sharings",
        {"circ_name": "isw_cube_noref"},
    ),
    "isw": ("ISW(x, y)", "eval_n_sharings", {"circ_name": "ISW", "n_inputs": 2}),
    "isw_y": (
        "ISW(y, x)",
        "eval_n_sharings",
        {"circ_name": "ISW", "n_inputs": 2, "sec_input": 1},
    ),
    "isw_xy": ("x|y: ISW(y, x)", "eval_mul_both", {}),
    "sh-add": (
        "Add(x, y)",
        "eval_n_sharings",
        {"circ_name": "sharewise_add", "n_inputs": 2},
    ),
}


@utils.pdt_cache.cache
def eval_circ_all_p(circ_name, k, e, d, ps, n_s_max, suff_thresh, use_copy, cum_tr):
    """Get the security level for a circuit for multiple p values."""
    _, f, kwargs = specialized_circuits[circ_name]
    gpdts = {
        circ: pdt_sampling.gpdt(circ, d, k, e, n_s_max, suff_thresh, use_copy, cum_tr)
        for circ in base_circuits[f][1](**kwargs)
    }
    print(
        "evaluating {} {}, d={}, n_s_max={}, suff_thresh={}, err={} use_copy={} cum_tr={}".format(
            circ_name, k, d, n_s_max, suff_thresh, e, use_copy, cum_tr
        )
    )

    def eval_fn(p):
        pdts = {circ: gpdt.instantiate(p) for circ, gpdt in gpdts.items()}
        return base_circuits[f][0](p, d=d, pdts=pdts, **kwargs)

    res_iter = utils.pmap(eval_fn, ps)
    return np.array(list(tqdm.tqdm(res_iter, total=len(ps), desc="All p")))
