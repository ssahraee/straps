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

import numpy as np

from ._straps_ext import PyProbeDistribution, PyPDT


class ProbeDistribution:
    """Proxy adapter for PyProbeDistribution.

    The main work is wire name adaptation: we support arbitrary types as wire
    names, while PyProbeDistribution only supports strings.
    """

    def __init__(self, wires, distr=None, full_pdt=False):
        if full_pdt:
            distr = np.eye(2 ** len(wires))
        if distr is None:
            distr = np.zeros((2 ** len(wires),), dtype=np.float64)
            distr[0] = 1.0
        self.single = len(distr.shape) == 1
        if len(distr.shape) == 1:
            distr = distr[:, np.newaxis]
        self.inner = PyProbeDistribution(self.w2ss(wires), distr)

    @staticmethod
    def w2s(wire):
        return "{}_{}".format(*wire)

    @staticmethod
    def w2ss(wires):
        return ["{}_{}".format(*w) for w in wires]

    def apply_op(self, inputs, outputs, pdt):
        if not isinstance(pdt, PyPDT):
            pdt = PyPDT.from_array(pdt)
        self.inner = self.inner.apply_op(self.w2ss(inputs), self.w2ss(outputs), pdt)
        return self

    def leak_wire(self, var, p):
        self.inner = self.inner.leak_wire(self.w2s(var), p)
        return self

    def split_wire(self, src, dest1, dest2):
        self.inner = self.inner.split_wire(
            self.w2s(src), self.w2s(dest1), self.w2s(dest2)
        )
        return self

    def distr(self, wires=None):
        if wires is not None:
            self.sort_wires(wires)
        if self.single:
            return self.inner.distr()[:, 0]
        else:
            return self.inner.distr()

    def sort_wires(self, wires):
        assert len(wires) == self.n_wires()
        for w in wires:
            self.wire_idx(w)
        self.inner.sort_wires([self.w2s(wire) for wire in wires])
        return self

    def wire_idx(self, wire):
        return self.inner.wire_idx(self.w2s(wire))

    def n_wires(self):
        return len(self.inner.wires())
