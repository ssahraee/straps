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


import math
import os

from matplotlib import pyplot as plt
import numpy as np

tikzplot = os.environ.get("STRAPS_TIKZPLOT") == "1"

if tikzplot:
    import tikzplotlib

from . import secfig

default_ds = list(range(1, 7))
small_ds = list(range(1, 6))

default_p = np.logspace(-4, 0, 40)

i = 0


def plot_save(
    fn, n_s_max=10 ** 8, suff_thresh=1000, ds=default_ds, err=1e-6, p=default_p
):
    global i
    n_s_max_s = "{:.0e}".format(n_s_max).replace("+0", "")
    suff_thresh_s = "{:.0e}".format(suff_thresh).replace("+0", "")
    if np.array_equal(p, default_p):
        fname = "{}_{}_{}".format(fn.replace("_", "-"), n_s_max_s, suff_thresh_s)
    else:
        fname = "{}-lp_{}_{}".format(fn.replace("_", "-"), n_s_max_s, suff_thresh_s)
    print("plot_save: ", fname)
    fig = plt.figure(str(i) + " " + fname)
    plt.title(fname)
    secfig.plot_fig(
        **secfig.data_fig(
            fn, ds=ds, n_s_max=n_s_max, suff_thresh=suff_thresh, err=err, p=p
        )
    )
    if tikzplot:
        tikzplotlib.save(
            filepath="figs/{}.tex".format(fname),
            figure=fig,
            axis_height="\\figureheight",
            axis_width="\\figurewidth",
            externalize_tables=True,
            override_externals=True,
            tex_relative_path_to_data="figs",
        )
    i += 1


if __name__ == "__main__":
    print("XOR vs ISW plot")
    plot_save("sh-add")
    plot_save("isw")

    print("suff_thresh plot")
    plot_save("isw", ds=small_ds, n_s_max=10 ** 7, suff_thresh=10)
    plot_save("isw", ds=small_ds, n_s_max=10 ** 7, suff_thresh=100)
    plot_save("isw", ds=small_ds, n_s_max=10 ** 7, suff_thresh=1000)

    print("n_s_max plot")
    plot_save("isw", ds=small_ds, n_s_max=10 ** 3)
    plot_save("isw", ds=small_ds, n_s_max=10 ** 5)
    plot_save("isw", ds=small_ds, n_s_max=10 ** 7)

    print("Int vs comp plot")
    plot_save("cube_isw_optref", ds=small_ds, n_s_max=2 * 10 ** 6)
    plot_save("int_cube_isw_optref", ds=small_ds, n_s_max=2 * 10 ** 6)

    print("Cube ref plot")
    plot_save("cube_isw_noref")
    plot_save("cube_isw_simpleref")
    plot_save("cube_isw_optref")

    print("AES S-box")
    plot_save("aes_sbox_noref")
    plot_save("aes_sbox_simpleref")
    plot_save("aes_sbox")
    plot_save("aes_sbox", p=np.logspace(-5, 0, 50))

    plt.show()
