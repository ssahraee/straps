
from straps import circuit_model
from straps import eval_circs, sh_pdt, pdt_sampling, secfig

def op_preamble(d, n_inputs):
    c = circuit_model.Circuit(d)
    output_sh = [
        c.var("out_{}".format(i), kind="output", port=(0, i)) for i in range(d)
    ]
    input_shs = [
        [c.var("in_{}_{}".format(j, i), kind="input", port=(j, i)) for i in range(d)]
        for j in range(n_inputs)
    ]
    return c, input_shs, output_sh

def mult_compression(d, c, x, y, z, r0, r1, m):
    #Compression step (same for each circuit)
    tmp = [{j: c.var("t_{}_{}".format(i, j)) for j in range(i + 1, d)} for i in range(d)] #Partial sums
    s = [[c.var("s_{}_{}".format(i, j)) for j in range(d)] for i in range(d)] 
    c_var = [[c.var("c_{}_{}".format(i, j)) for j in range(d)] for i in range(d)]
    for i in range(d):
        for j in range(i + 1, d):
            c.assign(s[i][j], r1[i][j])
            c.l_sum(tmp[i][j], (r1[i][j], m[i][j]))
            c.l_sum(s[j][i], (tmp[i][j], m[j][i]))
    for i in range(d):
        c.assign(s[i][i], m[i][i])
        c.assign(c_var[i][0], s[i][0])
        for j in range(1, d):
            c.l_sum(c_var[i][j], (c_var[i][j - 1], s[i][j]))
        c.assign(z[i], c_var[i][d - 1])

def snih_4_shares(d):
    
    if d != 4:
	    raise ValueError("This gadget works only with 4 shares.")
    c, (x, y), z = op_preamble(d, 2)

    
    # a fresh random
    r0 = [c.var("r_{}".format(i), kind="random") for i in range(d)]
    r1 = [
        {j: c.var(f"r1_{i}_{j}", kind="random") for j in range(i+1, d)}
        for i in range(d)
    ]

    
    # intermediate variables
    m = [[c.var("m_{}_{}".format(i, j)) for j in range(d)] for i in range(d)]
    u = [c.var("u_{}".format(i)) for i in range(d)]
    v = [c.var("v_{}".format(i)) for i in range(d)]

    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in y[:d//2]] for ix in x[:d//2]]
    
    #First iteration
    c.l_prod(m[0][0], ref_prods[0][0])
    c.l_prod(m[0][1], ref_prods[0][1])
    c.l_prod(m[1][0], ref_prods[1][0])
    c.l_prod(m[1][1], ref_prods[1][1])
    
    #First refresh
    c.l_sum(u[0], (r0[0], x[0]))
    c.l_sum(u[1], (r0[0], x[1]))
    c.l_sum(v[0], (r0[1], y[0]))
    c.l_sum(v[1], (r0[1], y[1]))
    
    # coeff product tuples
    ref_prods = [[(iu, iy) for iy in y[d//2:]] for iu in u[:d//2]]
    
    c.l_prod(m[0][2], ref_prods[0][0])
    c.l_prod(m[0][3], ref_prods[0][1])
    c.l_prod(m[1][2], ref_prods[1][0])
    c.l_prod(m[1][3], ref_prods[1][1])
    
    # coeff product tuples
    ref_prods = [[(ix, iv) for iv in v[:d//2]] for ix in x[d//2:]]
    
    c.l_prod(m[2][0], ref_prods[0][0])
    c.l_prod(m[2][1], ref_prods[0][1])
    c.l_prod(m[3][0], ref_prods[1][0])
    c.l_prod(m[3][1], ref_prods[1][1])
    
    #Second refresh
    c.l_sum(u[2], (r0[2], x[2]))
    c.l_sum(u[3], (r0[2], x[3]))
    c.l_sum(v[2], (r0[3], y[2]))
    c.l_sum(v[3], (r0[3], y[3]))
    
    # coeff product tuples
    ref_prods = [[(iu, iv) for iv in v[d//2:]] for iu in u[d//2:]]
    
    c.l_prod(m[2][2], ref_prods[0][0])
    c.l_prod(m[2][3], ref_prods[0][1])
    c.l_prod(m[3][2], ref_prods[1][0])
    c.l_prod(m[3][3], ref_prods[1][1])
    
    #Compression
    mult_compression(d, c, x, y, z, r0, r1, m)
            
    return c



# Integrate the gadget in the list of available gadgets:
from straps import simple_circuits
simple_circuits.all_circs["snih_4_shares"] = snih_4_shares


# Put in specialized_circuits a display name, and the name of your base_circuits entry
# (and a dict of optional parameter to your function).
eval_circs.specialized_circuits["snih_4_shares"] = ("snih_4_shares", "eval_n_sharings", {"circ_name": "snih_4_shares", "n_inputs": 2})
"""
# Then, you can use our top-level functions, e.g.
import numpy as np
from matplotlib import pyplot as plt
ds = [4] # number of shares
ps = np.logspace(-4, 0, 16) # parameter of the random probing model
e = 1e-6 # statistical confidence level
n_s_max = 10**5 # N_max
suff_thresh = 100 # N_t
secfig.plot_fig(**secfig.data_fig("snih_4_shares", ds, e, ps, n_s_max, suff_thresh))
secfig.plot_fig(**secfig.data_fig("isw", [2], e, ps, n_s_max, suff_thresh))
plt.show()
"""


