
import numpy as np

import matplotlib.pyplot as plt
import matplotlib.cm as cm
import matplotlib.colors

from . import ldt_sampling

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
    f = ldt_sampling.gpdt(gname, d=d, kind="ub", err=1e-6,n_s_max=10**5, suff_thresh=1000).to_array()
    res = np.argmax(f != 0.0, axis=0)
    colors = cm.get_cmap()._resample(np.max(res)+1).colors
    cmap = matplotlib.colors.ListedColormap(colors)
    fname = "mats/mat_{}_{}.png".format(gname, d)
    plt.figure(fname)
    plt.matshow(res, cmap=cmap)
    plt.colorbar()
    plt.title("PDT {} d={}, min nb probes".format(gname, d))
    plt.savefig(fname)
#plt.show()
