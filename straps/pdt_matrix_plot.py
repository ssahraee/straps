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

"""
Visual representation of PDT coefficients.
"""

import numpy as np

import matplotlib.pyplot as plt
import matplotlib.cm as cm
import matplotlib.colors

from . import pdt_sampling

plots = [
    ("ISW", 2),
    ("ISW", 3),
    ("ISW", 4),
    ("simpleref", 2),
    ("simpleref", 3),
    ("simpleref", 4),
    ("optref", 2),
    ("optref", 3),
    ("optref", 4),
]
for gname, d in plots:
    f = pdt_sampling.gpdt(
        gname, d=d, kind="ub", err=1e-6, n_s_max=10 ** 5, suff_thresh=1000
    ).to_array()
    res = np.argmax(f != 0.0, axis=0)
    colors = cm.get_cmap()._resample(np.max(res) + 1).colors
    cmap = matplotlib.colors.ListedColormap(colors)
    fname = "mats/mat_{}_{}.png".format(gname, d)
    plt.figure(fname)
    plt.matshow(res, cmap=cmap)
    plt.colorbar()
    plt.title("PDT {} d={}, min nb probes".format(gname, d))
    plt.savefig(fname)
# plt.show()
