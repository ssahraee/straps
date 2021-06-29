# Copyright 2018 Gaëtan Cassiers
# Copyright 2020 2021 UCLouvain
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

"""Arithmetic circuits for analysis.

This modules provides a simple interface to build arithmetic circuits and turn
them into a PyCompGraph.
"""

from ._straps_ext import PyCompGraph

KIND_MAP = {
    "input": PyCompGraph.VAR_KIND_INPUT,
    "random": PyCompGraph.VAR_KIND_RANDOM,
    "sum": PyCompGraph.VAR_KIND_SUM,
    "product": PyCompGraph.VAR_KIND_PRODUCT,
    "not": PyCompGraph.VAR_KIND_NOT,
}


class Circuit:
    def __init__(self, d=None):
        self.d = d
        self.vars = []
        self.assigns = []
        self.l_sums = []
        self.l_prods = []
        self.ngates = []
        self.input_ports = {}
        self.output_ports = {}

    def var(
        self, name, continuous=False, kind="intermediate", port=None, out_port=None
    ):
        k = "intermediate" if kind == "output" else kind
        v = Variable(name, len(self.vars), continuous, k)
        self.vars.append(v)
        if kind in ("input", "output"):
            assert port is not None
            # port is (in_port_id, id_of_var_in_port)
            if kind == "input":
                self.input_ports[v.idx] = port
            if kind == "output":
                self.output_ports[v.idx] = port
        if out_port is not None:
            self.output_ports[v.idx] = out_port
        return v

    @staticmethod
    def check_params_op(dest, ops):
        assert isinstance(dest, Variable)
        for op in ops:
            assert isinstance(op, Variable), "Not op: " + repr(op)

    def assign(self, dest, op):
        self.check_params_op(dest, (op,))
        self.assigns.append((dest, (op,)))

    def ngate(self, dest, op):
        self.check_params_op(dest, (op,))
        self.ngates.append((dest, (op,)))

    def l_sum(self, dest, ops):
        self.check_params_op(dest, ops)
        self.l_sums.append((dest, ops))

    def l_prod(self, dest, ops):
        self.check_params_op(dest, ops)
        self.l_prods.append((dest, ops))

    def simplified_assigns(self):
        """Remove all assigns such that assigned variables are merged."""
        # old to new
        map_vars = list(range(len(self.vars)))
        # new to olds
        rev_map_vars = list({x} for x in range(len(self.vars)))
        for dest, (op,) in self.assigns:
            map_vars[dest.idx] = map_vars[op.idx]
            rev_map_vars[map_vars[op.idx]] |= rev_map_vars[dest.idx]
            rev_map_vars[dest.idx] = None
        return self.var_mapped(rev_map_vars, map_vars)

    def sorted_vars(self):
        self.assert_all_nodes_have_input()
        copied_vars = [False] * len(self.vars)
        map_new_to_old = []

        def copy_var(i):
            assert not copied_vars[i]
            copied_vars[i] = True
            map_new_to_old.append(i)

        def copy_all_satisfying_predicate(pred):
            for i, v in enumerate(self.vars):
                if not copied_vars[i] and pred(i, v):
                    copy_var(i)

        for i, _ in sorted(
            ((i, v) for i, v in enumerate(self.vars) if v.kind == "input"),
            key=lambda iv: self.input_ports[iv[1].idx],
        ):
            copy_var(i)
        copy_all_satisfying_predicate(lambda _, v: v.kind == "random")
        ops_to_analyze = self.l_sums + self.l_prods + self.ngates + self.assigns
        while ops_to_analyze:
            new_ops_to_analyze = []
            for dest, ops in ops_to_analyze:
                assert not copied_vars[dest.idx]
                if all(copied_vars[op.idx] for op in ops):
                    copied_vars[dest.idx] = True
                    map_new_to_old.append(dest.idx)
                else:
                    new_ops_to_analyze.append((dest, ops))
            ops_to_analyze = new_ops_to_analyze
        assert all(copied_vars), "Not copied: {}".format(
            [v for i, v in enumerate(self.vars) if not copied_vars[i]]
        )
        assert len(map_new_to_old) == len(self.vars)
        map_old_to_new = {o: n for n, o in enumerate(map_new_to_old)}
        map_new_to_olds = [{o} for o in map_new_to_old]
        return self.var_mapped_new(map_new_to_olds, map_old_to_new, copy_assigns=True)

    def var_mapped(self, rev_map_vars, map_vars):
        self.assert_all_nodes_have_input()
        map_new_to_olds = [rv for rv in rev_map_vars if rv is not None]
        map_old_to_new = {o: n for n, olds in enumerate(map_new_to_olds) for o in olds}
        return self.var_mapped_new(map_new_to_olds, map_old_to_new)

    def assert_all_nodes_have_input(self):
        for v in self.vars:
            if v.kind == "intermediate":
                assert [
                    dest
                    for dest, _ in self.l_sums
                    + self.l_prods
                    + self.ngates
                    + self.assigns
                    if dest.idx == v.idx
                ], "{}".format(v)

    def var_mapped_new(self, map_new_to_olds, map_old_to_new, copy_assigns=False):
        self.assert_all_nodes_have_input()
        new_c = Circuit(self.d)
        for olds in map_new_to_olds:
            continuous = any(self.vars[ov].continuous for ov in olds)
            in_port = None
            if any(self.vars[ov].kind == "input" for ov in olds):
                kind = "input"
                in_port = next(
                    self.input_ports[ov] for ov in olds if self.vars[ov].kind == "input"
                )
            elif any(self.vars[ov].kind == "random" for ov in olds):
                kind = "random"
            elif any(self.vars[ov].kind == "intermediate" for ov in olds):
                kind = "intermediate"
            else:
                raise ValueError(kind)
            out_port = next(
                (self.output_ports[ov] for ov in olds if ov in self.output_ports), None
            )
            new_c.var(
                self.vars[next(iter(olds))].name, continuous, kind, in_port, out_port
            )
        copied_attrs = ["l_sums", "l_prods", "ngates"]
        if copy_assigns:
            copied_attrs.append("assigns")
        for attr in copied_attrs:
            for (dest, ops) in getattr(self, attr):
                new_dest_idx = map_old_to_new[dest.idx]
                a = new_c.vars[new_dest_idx]
            setattr(
                new_c,
                attr,
                [
                    (
                        new_c.vars[map_old_to_new[dest.idx]],
                        [new_c.vars[map_old_to_new[op.idx]] for op in ops],
                    )
                    for (dest, ops) in getattr(self, attr)
                ],
            )
        new_c.assert_all_nodes_have_input()
        return new_c, map_old_to_new

    def to_comp_graph(self):
        sc, _ = self.simplified_assigns()
        sc, _ = sc.sorted_vars()
        comp_graph = [None for _ in sc.vars]
        sum_ops = [
            (dest, KIND_MAP["sum"], [op1.idx, op2.idx])
            for dest, (op1, op2) in sc.l_sums
        ]
        prod_ops = [
            (dest, KIND_MAP["product"], [op1.idx, op2.idx])
            for dest, (op1, op2) in sc.l_prods
        ]
        not_ops = [(dest, KIND_MAP["not"], [op1.idx]) for dest, (op1,) in sc.ngates]
        for dest, op_kind, opsidx in sum_ops + prod_ops + not_ops:
            assert comp_graph[dest.idx] is None
            assert dest.kind == "intermediate"
            comp_graph[dest.idx] = (op_kind, opsidx, dest.name)
        for var in sc.vars:
            if var.kind in ("random", "input"):
                assert comp_graph[var.idx] is None
                comp_graph[var.idx] = (KIND_MAP[var.kind], [], var.name)
        for i, x in enumerate(comp_graph):
            assert x is not None
        in_ports = [sc.input_ports.get(v.idx) for v in sc.vars]
        out_ports = [sc.output_ports.get(v.idx) for v in sc.vars]
        return PyCompGraph(
            comp_graph,
            in_ports,
            out_ports,
            self.d,
            len(sc.input_ports) // self.d,
            len(sc.output_ports) // self.d,
        )

    def in_ports(self):
        return set(x[0] for x in self.input_ports.values())

    def out_ports(self):
        return set(x[0] for x in self.output_ports.values())


class Variable:
    def __init__(self, name, idx, continuous, kind):
        self.name = name
        self.idx = idx
        self.continuous = continuous
        self.kind = kind

    def __repr__(self):
        return "Variable(name={}, idx={}, continuous={}, kind={})".format(
            self.name, self.idx, self.continuous, self.kind
        )
