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


"""Base circuits for which we compute the PDTs."""

from . import circuit_model


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


def sharewise_add(d):
    c, (sh_x, sh_y), sh_z = op_preamble(d, 2)
    for i in range(d):
        c.l_sum(sh_z[i], (sh_x[i], sh_y[i]))
    return c


def square(d):
    c, (sh_x,), sh_z = op_preamble(d, 1)
    for i in range(d):
        c.l_prod(sh_z[i], (sh_x[i], sh_x[i]))
    return c


def isw_mul(d):
    c, (sh_x, sh_y), sh_z = op_preamble(d, 2)
    isw_inner(d, c, sh_x, sh_y, sh_z)
    return c


def mul_matrix(d):
    c, (sh_x, sh_y), sh_z = op_preamble(d, 2)
    prods = [[c.var("p_{}_{}".format(i, j)) for i in range(d)] for j in range(d)]
    for i in range(d):
        for j in range(d):
            c.l_prod(prods[i][j], (sh_x[i], sh_y[j]))
    for i in range(d):
        c.assign(sh_z[i], prods[i][i])
    return c


def isw_cube_ref(d, ref_name="opt_ref"):
    c, (sh_x,), sh_z = op_preamble(d, 1)
    sh_y, sh_w = [], []
    for i in range(d):
        sh_y.append(c.var("y_{}".format(i), kind="intermediate"))
        sh_w.append(c.var("w_{}".format(i), kind="intermediate"))
    if ref_name == "noref":
        for sx, sy in zip(sh_x, sh_y):
            c.assign(sy, sx)
    else:
        refs[ref_name](c, sh_x, sh_y)
    for sy, sw in zip(sh_y, sh_w):
        c.l_sum(sw, (sy, sy))
    isw_inner(d, c, sh_x, sh_w, sh_z)
    return c


def isw_inner(d, c, x, y, z):
    # product matrix
    ref_prods = [[(ix, iy) for iy in y] for ix in x]

    p = [[c.var("p_{}_{}".format(i, j)) for j in range(d)] for i in range(d)]
    for i in range(d):
        for j in range(d):
            c.l_prod(p[i][j], ref_prods[i][j])
    r = [
        {j: c.var("r_{}_{}".format(i, j), kind="random") for j in range(i + 1, d)}
        for i in range(d)
    ]
    t = [{j: c.var("t_{}_{}".format(i, j)) for j in range(i + 1, d)} for i in range(d)]
    s = [[c.var("s_{}_{}".format(i, j)) for j in range(d)] for i in range(d)]
    c_var = [[c.var("c_{}_{}".format(i, j)) for j in range(d)] for i in range(d)]
    for i in range(d):
        for j in range(i + 1, d):
            c.assign(s[i][j], r[i][j])
            c.l_sum(t[i][j], (r[i][j], p[i][j]))
            c.l_sum(s[j][i], (t[i][j], p[j][i]))
    for i in range(d):
        c.assign(s[i][i], p[i][i])
        c.assign(c_var[i][0], s[i][0])
        for j in range(1, d):
            c.l_sum(c_var[i][j], (c_var[i][j - 1], s[i][j]))
        c.assign(z[i], c_var[i][d - 1])


def opt_ref(circuit, inputs, outputs):
    d = len(inputs)
    if d == 1 or d == 2:
        return simple_ref(circuit, inputs, outputs)
    elif d == 3:
        r0 = circuit.var("r_0", kind="random")
        r1 = circuit.var("r_1", kind="random")
        t = circuit.var("t")
        circuit.l_sum(t, (r0, r1))
        circuit.l_sum(outputs[0], (inputs[0], r0))
        circuit.l_sum(outputs[1], (inputs[1], r1))
        circuit.l_sum(outputs[2], (inputs[2], t))
        return outputs
    elif d == 4 or d == 5:
        return rot_ref(circuit, inputs, outputs)
    elif d == 6:
        randoms = [circuit.var("r_{}".format(i), kind="random") for i in range(d)]
        temps = [circuit.var("temp_{}".format(i)) for i in range(d)]
        for r1, r2, t in zip(randoms, randoms[1:] + [randoms[0]], temps):
            circuit.l_sum(t, (r1, r2))
        rt = circuit.var("rt", kind="random")
        t0 = circuit.var("t0")
        t3 = circuit.var("t3")
        circuit.l_sum(t0, (temps[0], rt))
        circuit.l_sum(t3, (temps[3], rt))
        temps[0] = t0
        temps[3] = t3
        for i, t, o in zip(inputs, temps, outputs):
            circuit.l_sum(o, (i, t))
        return outputs
    else:
        raise NotImplemented("TODO")


def simple_ref(circuit, inputs, outputs):
    d = len(inputs)
    r = [circuit.var("r_{}".format(i), kind="random") for i in range(d - 1)]
    if d == 1:
        circuit.assign(outputs[0], inputs[0])
    elif d == 2:
        circuit.l_sum(outputs[0], (inputs[0], r[0]))
        circuit.l_sum(outputs[1], (inputs[1], r[0]))
    elif d >= 3:
        t = [circuit.var("t_{}".format(i)) for i in range(d - 2)]
        for i in range(d - 1):
            circuit.l_sum(outputs[i], (inputs[i], r[i]))
        circuit.l_sum(t[0], (inputs[-1], r[0]))
        for i in range(1, d - 2):
            circuit.l_sum(t[i], (t[i - 1], r[i]))
        circuit.l_sum(outputs[d - 1], (t[d - 3], r[d - 2]))
    return outputs


def rot_ref(circuit, inputs, outputs):
    d = len(inputs)
    if d == 1:
        circuit.assign(outputs[0], inputs[0])
    else:
        randoms = [circuit.var("r_{}".format(i), kind="random") for i in range(d)]
        temps = [circuit.var("temp_{}".format(i)) for i in range(d)]
        for r1, r2, t in zip(randoms, randoms[1:] + [randoms[0]], temps):
            circuit.l_sum(t, (r1, r2))
        for i, t, o in zip(inputs, temps, outputs):
            circuit.l_sum(o, (i, t))
    return outputs


refs = {
    "simple_ref": simple_ref,
    "opt_ref": opt_ref,
}


def ref_wrapper(d, ref):
    c, (sh_x,), sh_z = op_preamble(d, 1)
    ref(circuit=c, inputs=sh_x, outputs=sh_z)
    return c


# map 'name' -> builder(d)
all_circs = {
    "ISW": isw_mul,
    "simpleref": lambda d: ref_wrapper(d, simple_ref),
    "optref": lambda d: ref_wrapper(d, opt_ref),
    "sharewise_add": sharewise_add,
    "square": square,
    "mul_matrix": mul_matrix,
    "isw_cube_opt": lambda d: isw_cube_ref(d, "opt_ref"),
    "isw_cube_simple": lambda d: isw_cube_ref(d, "simple_ref"),
    "isw_cube_noref": lambda d: isw_cube_ref(d, "noref"),
}


def circ2pcg(name, d):
    return all_circs[name](d).to_comp_graph()


if __name__ == "__main__":
    for k in all_circs.keys():
        print("testing", k)
        for d in range(1, 7):
            circ2pcg(k, d)
