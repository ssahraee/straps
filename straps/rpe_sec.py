# Copyright 2021 UCLouvain
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""
Plot the security versus number of shares of SOTA random probing security
papers.
"""

import numpy as np
import scipy.special


### First paper optimized gadgets with 3 shares
# "Random Probing Security: Verification, Composition, Expansion and New
# Constructions", Appendix D

# G^2_add gadget
# G_add f_1^1 c_i: [0, 3, 118, 2457, 34998]
# G_add f_2^1 c_i: [0, 3, 106, 2035, 27812]
# G_add f_12^1 c_i: [0, 0, 0, 69, 3034]
# G_add f_1^2 c_i: [0, 3, 118, 2403, 29859]
# G_add f_2^2 c_i: [0, 3, 106, 2007, 22079]
# G_add f_12^2 c_i: [0, 0, 0, 9, 600]
# G_add s: 36
p1_g_add = {
    "s": 36,
    "f": (
        ([0, 3, 118, 2457, 34998], [0, 3, 106, 2035, 27812], [0, 0, 0, 69, 3034]),
        ([0, 3, 118, 2403, 29859], [0, 3, 106, 2007, 22079], [0, 0, 0, 9, 600]),
    ),
}

# G_mul f_1^1 c_i: [0, 3, 1232, 60940, 1653719]
# G_mul f_2^1 c_i: [0, 7, 1688, 74662, 2152987]
# G_mul f_12^1 c_i: [0, 0, 62, 5300, 291603]
# G_mul f_1^2 c_i: [0, 3, 1254, 42135, 1428624]
# G_mul f_2^2 c_i: [0, 11, 2135, 47322, 1437774]
# G_mul f_12^2 c_i: [0, 0, 83, 4248, 255461]
# G_mul s: 97
p1_g_mul = {
    "s": 97,
    "f": (
        (
            [0, 3, 1232, 60940, 1653719],
            [0, 7, 1688, 74662, 2152987],
            [0, 0, 62, 5300, 291603],
        ),
        (
            [0, 3, 1254, 42135, 1428624],
            [0, 11, 2135, 47322, 1437774],
            [0, 0, 83, 4248, 255461],
        ),
    ),
}

# G_copy f_11 c_i: [0, 33, 1137, 16812, 145288, 852472, 3750849, 13073855, 37574146, 91573962, 192726070, 354263297, 572852089, 818662608, 1037103082, 1166786707, 1166799413, 1037157725, 818809139, 573166437, 354817320, 193536720, 92561040, 38567100, 13884156, 4272048, 1107568, 237336, 40920, 5456, 528, 33, 1]
# G_copy f_12 c_i: [0, 30, 1285, 19887, 166695, 951201, 4021599, 13567630, 38231896, 92255103, 193295461, 354654683, 573074084, 818765733, 1037141693, 1166798076, 1166801950, 1037158129, 818809180, 573166439, 354817320, 193536720, 92561040, 38567100, 13884156, 4272048, 1107568, 237336, 40920, 5456, 528, 33, 1]
# G_copy f_21 c_i: [0, 30, 1285, 19887, 166695, 951201, 4021599, 13567630, 38231896, 92255103, 193295461, 354654683, 573074084, 818765733, 1037141693, 1166798076, 1166801950, 1037158129, 818809180, 573166439, 354817320, 193536720, 92561040, 38567100, 13884156, 4272048, 1107568, 237336, 40920, 5456, 528, 33, 1]
# G_copy f_22 c_i: [0, 27, 1433, 23538, 188460, 1016149, 4150387, 13760724, 38465921, 92491608, 193496624, 354798258, 573159259, 818807160, 1037157912, 1166803059, 1166803107, 1037158320, 818809200, 573166440, 354817320, 193536720, 92561040, 38567100, 13884156, 4272048, 1107568, 237336, 40920, 5456, 528, 33, 1]
# G_copy s: 33


p1_g_copy = {
    "s": 33,
    "f": (
        [
            0,
            33,
            1137,
            16812,
            145288,
            852472,
            3750849,
            13073855,
            37574146,
            91573962,
            192726070,
            354263297,
            572852089,
            818662608,
            1037103082,
            1166786707,
            1166799413,
            1037157725,
            818809139,
            573166437,
            354817320,
            193536720,
            92561040,
            38567100,
            13884156,
            4272048,
            1107568,
            237336,
            40920,
            5456,
            528,
            33,
            1,
        ],
        [
            0,
            30,
            1285,
            19887,
            166695,
            951201,
            4021599,
            13567630,
            38231896,
            92255103,
            193295461,
            354654683,
            573074084,
            818765733,
            1037141693,
            1166798076,
            1166801950,
            1037158129,
            818809180,
            573166439,
            354817320,
            193536720,
            92561040,
            38567100,
            13884156,
            4272048,
            1107568,
            237336,
            40920,
            5456,
            528,
            33,
            1,
        ],
        [
            0,
            30,
            1285,
            19887,
            166695,
            951201,
            4021599,
            13567630,
            38231896,
            92255103,
            193295461,
            354654683,
            573074084,
            818765733,
            1037141693,
            1166798076,
            1166801950,
            1037158129,
            818809180,
            573166439,
            354817320,
            193536720,
            92561040,
            38567100,
            13884156,
            4272048,
            1107568,
            237336,
            40920,
            5456,
            528,
            33,
            1,
        ],
        [
            0,
            27,
            1433,
            23538,
            188460,
            1016149,
            4150387,
            13760724,
            38465921,
            92491608,
            193496624,
            354798258,
            573159259,
            818807160,
            1037157912,
            1166803059,
            1166803107,
            1037158320,
            818809200,
            573166440,
            354817320,
            193536720,
            92561040,
            38567100,
            13884156,
            4272048,
            1107568,
            237336,
            40920,
            5456,
            528,
            33,
            1,
        ],
    ),
}

p1_gadgets = {
    "add": p1_g_add,
    "mul": p1_g_mul,
    "copy": p1_g_copy,
    "d": 3,
}

# f(p) = \sum_{i=0}^s c_i p^i (1-p)^{s-i} with c_i <= C_s^i
def sum_coef(p, coefs, s):
    u_coefs = [scipy.special.binom(s, i) for i in range(s + 1)]
    for i, c in enumerate(coefs):
        u_coefs[i] = c
    return sum(p ** i * (1 - p) ** (s - i) * c for i, c in enumerate(u_coefs))


# 2-input gadgets:
# f_max(p) = max(f_1^1(p), f_1^2(p), f_2^1(p), f_2^2(p), sqrt(f_12^1(p)), sqrt(f_12^2(p))
def fmax_2in(p, g):
    return np.amax(
        [
            sum_coef(p, g["f"][0][0], g["s"]),
            sum_coef(p, g["f"][1][0], g["s"]),
            sum_coef(p, g["f"][0][1], g["s"]),
            sum_coef(p, g["f"][1][1], g["s"]),
            np.sqrt(sum_coef(p, g["f"][0][2], g["s"])),
            np.sqrt(sum_coef(p, g["f"][1][2], g["s"])),
        ],
        axis=0,
    )


def circuit_f(p, circuit):
    fmax_add = fmax_2in(p, circuit["add"])
    fmax_mul = fmax_2in(p, circuit["mul"])
    # copy gadget
    # f_max(p) = max(f^1(p), f^2(p), f^12(p), f^21(p))
    fmax_copy = np.amax(
        [sum_coef(p, circuit["copy"]["f"][i], circuit["copy"]["s"]) for i in range(4)],
        axis=0,
    )
    # f_max: f_max over all gadgets
    f_max = np.amax([fmax_add, fmax_mul, fmax_copy], axis=0)
    # f(p) = f_max(p) + 3/2*f_max(p)**2
    f = f_max + 1.5 * f_max ** 2
    return f


# circuit size: number of gadgets in the circuit
# k = number of expansions
# sec_circuit = circuit_size*2*(f(p)**k)
def sec_circuit(p, circuit_gadgets, circuit_size, k):
    return np.minimum(1, circuit_size * 2 * circuit_f(p, circuit_gadgets) ** k)


def circuit_n_shares(circuit_gadgets, k):
    return circuit_gadgets["d"] ** k


p = np.logspace(-5, 0, 50)

# 11 copy
# 11 mul
aes_inversion_circuit_size = 22

# Paper: TODO
def df_sec(d, p, s):
    return np.minimum(1, s * (32 * d * p + 4 * d * np.sqrt(3 * p)) ** d)


cgs = {
    "RPE": [
        ("d=3", lambda p, s: sec_circuit(p, p1_gadgets, s, 1)),
        ("d=9", lambda p, s: sec_circuit(p, p1_gadgets, s, 2)),
        ("d=27", lambda p, s: sec_circuit(p, p1_gadgets, s, 3)),
    ],
    "DFZ": [
        ("d=3", lambda p, s: df_sec(3, p, s)),
        ("d=9", lambda p, s: df_sec(9, p, s)),
        ("d=27", lambda p, s: df_sec(27, p, s)),
    ],
}

import matplotlib.pyplot as plt
import tikzplotlib

for paper_name, cg in cgs.items():
    fig = plt.figure()
    for name, fn in cg:
        plt.loglog(p, fn(p, aes_inversion_circuit_size), label=name)
    plt.legend()
    plt.xlabel("p")
    plt.ylabel("security level")
    if True:
        tikzplotlib.save(
            filepath="figs/{}.tex".format(paper_name),
            figure=fig,
            axis_height="\\figureheight",
            axis_width="\\figurewidth",
            externalize_tables=True,
            override_externals=True,
            tex_relative_path_to_data="figs",
        )

# plt.show()
