from straps import circuit_model
from straps import eval_circs, sh_pdt, pdt_sampling, secfig, simple_circuits
import numpy as np
from matplotlib import pyplot as plt

def test_isw(d):
    c, (x, y), z = simple_circuits.op_preamble(d,2)
    ref_prods = [[(ix, iy) for iy in y] for ix in x]

    p = [[c.var("p_{}_{}".format(i, j)) for j in range(d)] for i in range(d)]
    for i in range(d):
        for j in range(d):
            c.l_prod(p[i][j], ref_prods[i][j])
    r = [
        {j: c.var("r_{}_{}".format(i, j), kind="random") for j in range(i + 1, d)}
        for i in range(d)
    ]
    
    #compression
    tmp = c.var("tmp")
    s = [[c.var("s_{}_{}".format(i, j)) for j in range(d)] for i in range(d)]
    c_var = [[c.var("c_{}_{}".format(i, j)) for j in range(d)] for i in range(d)]
    for i in range(d):
        for j in range(i + 1, d):
            c.assign(s[i][j], r[i][j])
            c.l_sum(tmp, (r[i][j], p[i][j]))
            c.l_sum(s[j][i], (tmp, p[j][i]))
    for i in range(d):
        c.assign(s[i][i], p[i][i])
        c.assign(c_var[i][0], s[i][0])
        for j in range(1, d):
            c.l_sum(c_var[i][j], (c_var[i][j - 1], s[i][j]))
        c.assign(z[i], c_var[i][d - 1])
    return 0

# Integrate the gadget in the list of available gadgets:

simple_circuits.all_circs["test_isw"] = test_isw

eval_circs.specialized_circuits["test_isw"] = ("test_isw", "eval_n_sharings", {"circ_name": "test_isw", "n_inputs": 2})



# Then, you can use our top-level functions, e.g.

ds = [4] # number of shares
ps = np.logspace(-4, 0, 16) # parameter of the random probing model
e = 1e-6 # statistical confidence level
n_s_max = 10**5 # N_max
suff_thresh = 100 # N_t
secfig.plot_fig(**secfig.data_fig("test_isw", ds, e, ps, n_s_max, suff_thresh))
