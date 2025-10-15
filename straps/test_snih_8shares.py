from straps import circuit_model
from straps import eval_circs, sh_pdt, pdt_sampling, secfig
import test_snih_4shares

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


def snih_8_shares(d):
    
    if d != 8:
        raise ValueError("This gadget works only with 8 shares.")
    c, (x, y), z = op_preamble(d, 2)

    
    # a fresh random
    r0 = [c.var("r_{}".format(i), kind="random") for i in range(40)]
    r1 = [
        {j: c.var(f"r_{i}_{j}", kind="random") for j in range(i+1, d)}
        for i in range(d)
    ]

    
    # intermediate variables
    m = [[c.var("m_{}_{}".format(i, j)) for j in range(d)] for i in range(d)]
    u0 = [c.var("u0_{}".format(i)) for i in range(d)]
    v0 = [c.var("v0_{}".format(i)) for i in range(d)]
    u1 = [c.var("u1_{}".format(i)) for i in range(d)] #replacing s 
    v1 = [c.var("v1_{}".format(i)) for i in range(d)] # replacing t
    u2 = [c.var("u2_{}".format(i)) for i in range(d)]
    v2 = [c.var("v2_{}".format(i)) for i in range(d)]
    u3 = [c.var("u3_{}".format(i)) for i in range(d)]
    v3 = [c.var("v3_{}".format(i)) for i in range(d)]
    u4 = [c.var("u4_{}".format(i)) for i in range(d)]
    v4 = [c.var("v4_{}".format(i)) for i in range(d)]    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in y[:d//4]] for ix in x[:d//4]]
    
    #First iteration
    c.l_prod(m[0][0], ref_prods[0][0])
    c.l_prod(m[0][1], ref_prods[0][1])
    c.l_prod(m[1][0], ref_prods[1][0])
    c.l_prod(m[1][1], ref_prods[1][1])
    
    #First refresh
    c.l_sum(u0[0], (r0[0], x[0]))
    c.l_sum(u0[1], (r0[0], x[1]))
    c.l_sum(v0[0], (r0[1], y[0]))
    c.l_sum(v0[1], (r0[1], y[1]))
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in y[d//4:d//2]] for ix in u0[:d//4]]
    
    c.l_prod(m[0][2], ref_prods[0][0])
    c.l_prod(m[0][3], ref_prods[0][1])
    c.l_prod(m[1][2], ref_prods[1][0])
    c.l_prod(m[1][3], ref_prods[1][1])
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v0[:d//4]] for ix in x[d//4:d//2]]
    
    c.l_prod(m[2][0], ref_prods[0][0])
    c.l_prod(m[2][1], ref_prods[0][1])
    c.l_prod(m[3][0], ref_prods[1][0])
    c.l_prod(m[3][1], ref_prods[1][1])
    
    #Second refresh
    c.l_sum(u0[2], (r0[2], x[2]))
    c.l_sum(u0[3], (r0[2], x[3]))
    c.l_sum(v0[2], (r0[3], y[2]))
    c.l_sum(v0[3], (r0[3], y[3]))
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v0[d//4:d//2]] for ix in u0[d//4:d//2]]
    
    c.l_prod(m[2][2], ref_prods[0][0])
    c.l_prod(m[2][3], ref_prods[0][1])
    c.l_prod(m[3][2], ref_prods[1][0])
    c.l_prod(m[3][3], ref_prods[1][1])

    
    #Second refresh
    c.l_sum(u1[0], (r0[4], x[0]))
    c.l_sum(u1[2], (r0[4], x[2]))
    c.l_sum(u1[1], (r0[5], x[1]))
    c.l_sum(u1[3], (r0[5], x[3]))
    
    c.l_sum(u2[0], (r0[6], u1[0]))
    c.l_sum(u2[1], (r0[6], u1[1]))
    c.l_sum(u2[2], (r0[7], u1[2]))
    c.l_sum(u2[3], (r0[7], u1[3]))
    
    c.l_sum(u3[0], (r0[8], u2[0]))
    c.l_sum(u3[2], (r0[8], u2[2]))
    c.l_sum(u3[1], (r0[9], u2[1]))
    c.l_sum(u3[3], (r0[9], u2[3]))
    
    c.l_sum(v1[0], (r0[10], y[0]))
    c.l_sum(v1[2], (r0[10], y[2]))
    c.l_sum(v1[1], (r0[11], y[1]))
    c.l_sum(v1[3], (r0[11], y[3]))
    
    c.l_sum(v2[0], (r0[12], v1[0]))
    c.l_sum(v2[1], (r0[12], v1[1]))
    c.l_sum(v2[2], (r0[13], v1[2]))
    c.l_sum(v2[3], (r0[13], v1[3]))
    
    c.l_sum(v3[0], (r0[14], v2[0]))
    c.l_sum(v3[2], (r0[14], v2[2]))
    c.l_sum(v3[1], (r0[15], v2[1]))
    c.l_sum(v3[3], (r0[15], v2[3]))
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in y[d//2:3*d//4]] for ix in u3[:d//4]]
    
    #prods
    c.l_prod(m[0][4], ref_prods[0][0])
    c.l_prod(m[0][5], ref_prods[0][1])
    c.l_prod(m[1][4], ref_prods[1][0])
    c.l_prod(m[1][5], ref_prods[1][1])
    
    c.l_sum(u4[0], (r0[16], u3[0]))
    c.l_sum(u4[1], (r0[16], u3[1]))
    
    c.l_sum(v0[4], (r0[17], y[4]))
    c.l_sum(v0[5], (r0[17], y[5]))
    
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in y[3*d//4:]] for ix in u4[:d//4]]
    
    #prods
    c.l_prod(m[0][6], ref_prods[0][0])
    c.l_prod(m[0][7], ref_prods[0][1])
    c.l_prod(m[1][6], ref_prods[1][0])
    c.l_prod(m[1][7], ref_prods[1][1])
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v0[d//2:3*d//4]] for ix in u3[d//4:d//2]]
    
    #prods
    c.l_prod(m[2][4], ref_prods[0][0])
    c.l_prod(m[2][5], ref_prods[0][1])
    c.l_prod(m[3][4], ref_prods[1][0])
    c.l_prod(m[3][5], ref_prods[1][1])

    
    #a2 = s2 + r18
    c.l_sum(u4[2], (r0[18], u3[2]))
    c.l_sum(u4[3], (r0[18], u3[3]))
    
    c.l_sum(v0[6], (r0[19], y[6]))
    c.l_sum(v0[7], (r0[19], y[7]))
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v0[3*d//4:]] for ix in u4[d//4:d//2]]
    
    #prods
    c.l_prod(m[2][6], ref_prods[0][0])
    c.l_prod(m[2][7], ref_prods[0][1])
    c.l_prod(m[3][6], ref_prods[1][0])
    c.l_prod(m[3][7], ref_prods[1][1])
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v3[:d//4]] for ix in x[d//2:3*d//4]]
    
    #prods
    c.l_prod(m[4][0], ref_prods[0][0])
    c.l_prod(m[4][1], ref_prods[0][1])
    c.l_prod(m[5][0], ref_prods[1][0])
    c.l_prod(m[5][1], ref_prods[1][1])
    
    c.l_sum(u0[4], (r0[20], x[4]))
    c.l_sum(u0[5], (r0[20], x[5]))
    
    c.l_sum(v4[0], (r0[21], v3[0]))
    c.l_sum(v4[1], (r0[21], v3[1]))
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v3[d//4:d//2]] for ix in u0[d//2:3*d//4]]
    
    #prods
    c.l_prod(m[4][2], ref_prods[0][0])
    c.l_prod(m[4][3], ref_prods[0][1])
    c.l_prod(m[5][2], ref_prods[1][0])
    c.l_prod(m[5][3], ref_prods[1][1])
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v4[:d//4]] for ix in x[3*d//4:]]
    
    #prods
    c.l_prod(m[6][0], ref_prods[0][0])
    c.l_prod(m[6][1], ref_prods[0][1])
    c.l_prod(m[7][0], ref_prods[1][0])
    c.l_prod(m[7][1], ref_prods[1][1])

    c.l_sum(u0[6], (r0[22], x[6]))
    c.l_sum(u0[7], (r0[22], x[7]))
    
    c.l_sum(v4[2], (r0[24], v3[2]))
    c.l_sum(v4[3], (r0[24], v3[3]))

    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v4[d//4:d//2]] for ix in u0[3*d//4:]]
    
    #prods
    c.l_prod(m[6][2], ref_prods[0][0])
    c.l_prod(m[6][3], ref_prods[0][1])
    c.l_prod(m[7][2], ref_prods[1][0])
    c.l_prod(m[7][3], ref_prods[1][1])
    
    
    c.l_sum(u1[4], (r0[24], x[4]))
    c.l_sum(u1[6], (r0[24], x[6]))
    c.l_sum(u1[5], (r0[25], x[5]))
    c.l_sum(u1[7], (r0[25], x[7]))
    
    c.l_sum(u2[4], (r0[26], u1[4]))
    c.l_sum(u2[5], (r0[26], u1[5]))
    c.l_sum(u2[6], (r0[27], u1[6]))
    c.l_sum(u2[7], (r0[27], u1[7]))
    
    c.l_sum(u3[4], (r0[28], u2[4]))
    c.l_sum(u3[6], (r0[28], u2[6]))
    c.l_sum(u3[5], (r0[29], u2[5]))
    c.l_sum(u3[7], (r0[29], u2[7]))
    
    c.l_sum(v1[4], (r0[30], y[4]))
    c.l_sum(v1[6], (r0[30], y[6]))
    c.l_sum(v1[5], (r0[31], y[5]))
    c.l_sum(v1[7], (r0[31], y[7]))
    
    c.l_sum(v2[4], (r0[32], v1[4]))
    c.l_sum(v2[5], (r0[32], v1[5]))
    c.l_sum(v2[6], (r0[33], v1[6]))
    c.l_sum(v2[7], (r0[33], v1[7]))
    
    c.l_sum(v3[4], (r0[34], v2[4]))
    c.l_sum(v3[6], (r0[34], v2[6]))
    c.l_sum(v3[5], (r0[35], v2[5]))
    c.l_sum(v3[7], (r0[35], v2[7]))
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v3[d//2:3*d//4]] for ix in u3[d//2:3*d//4]]
    
    #prods
    c.l_prod(m[4][4], ref_prods[0][0])
    c.l_prod(m[4][5], ref_prods[0][1])
    c.l_prod(m[5][4], ref_prods[1][0])
    c.l_prod(m[5][5], ref_prods[1][1])
  
    c.l_sum(u4[4], (r0[36], u3[4]))
    c.l_sum(u4[5], (r0[36], u3[5]))
    
    c.l_sum(v4[4], (r0[37], v3[4]))
    c.l_sum(v4[5], (r0[37], v3[5]))
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v3[3*d//4:]] for ix in u4[d//2:3*d//4]]
    
    #prods
    c.l_prod(m[4][6], ref_prods[0][0])
    c.l_prod(m[4][7], ref_prods[0][1])
    c.l_prod(m[5][6], ref_prods[1][0])
    c.l_prod(m[5][7], ref_prods[1][1])
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v4[d//2:3*d//4]] for ix in u3[3*d//4:]]
    
    #prods
    c.l_prod(m[6][4], ref_prods[0][0])
    c.l_prod(m[6][5], ref_prods[0][1])
    c.l_prod(m[7][4], ref_prods[1][0])
    c.l_prod(m[7][5], ref_prods[1][1])
    
    c.l_sum(u4[6], (r0[38], u3[6]))
    c.l_sum(u4[7], (r0[38], u3[7]))
    
    c.l_sum(v4[6], (r0[39], v3[6]))
    c.l_sum(v4[7], (r0[39], v3[7]))
    
    # coeff product tuples
    ref_prods = [[(ix, iy) for iy in v4[3*d//4:]] for ix in u4[3*d//4:]]
    
    #prods
    c.l_prod(m[6][6], ref_prods[0][0])
    c.l_prod(m[6][7], ref_prods[0][1])
    c.l_prod(m[7][6], ref_prods[1][0])
    c.l_prod(m[7][7], ref_prods[1][1])

    #Compression step (same for each circuit)
    mult_compression(d, c, x, y, z, r0, r1, m)
    return c


    
   
   # Integrate the gadget in the list of available gadgets:
from straps import simple_circuits
simple_circuits.all_circs["snih_8_shares"] = snih_8_shares
simple_circuits.all_circs["snih_4_shares"] = test_snih_4shares.snih_4_shares
eval_circs.specialized_circuits["snih_8_shares"] = ("snih_8_shares", "eval_n_sharings", {"circ_name": "snih_8_shares", "n_inputs": 2})
eval_circs.specialized_circuits["snih_4_shares"] = ("snih_4_shares", "eval_n_sharings", {"circ_name": "snih_8_shares", "n_inputs": 2})

# Then, you can use our top-level functions, e.g.
import numpy as np
from matplotlib import pyplot as plt
ds = [8] # number of shares
ps = np.logspace(-4, 0, 16) # parameter of the random probing model
e = 1e-6 # statistical confidence level
n_s_max = 10**5 # N_max
suff_thresh = 100 # N_t
secfig.plot_fig(**secfig.data_fig("snih_8_shares", [8], e, ps, n_s_max, suff_thresh))
secfig.plot_fig(**secfig.data_fig("snih_4_shares", [4], e, ps, n_s_max, suff_thresh))
secfig.plot_fig(**secfig.data_fig("isw", [2], e, ps, n_s_max, suff_thresh))
plt.show()

    
