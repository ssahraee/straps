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


import itertools as it

from .probe_distr import ProbeDistribution


class ShPd(ProbeDistribution):
    """Probe distribution of multiple sharings.

    All the operations are performed over sharings.
    """

    def __init__(self, sharings, n_shares, distr=None):
        self.n_shares = n_shares
        super(ShPd, self).__init__(list(it.product(sharings, range(n_shares))), distr)

    def op(self, dest, srcs, pdt):
        return self.apply_op(
            list(it.product(srcs, range(self.n_shares))),
            [(dest, i) for i in range(self.n_shares)],
            pdt,
        )

    def split_sharing(self, src, dest1, dest2):
        for i in range(self.n_shares):
            self.split_wire((src, i), (dest1, i), (dest2, i))
        return self

    def lin_op(self, sharing, p):
        for i in range(self.n_shares):
            self.leak_wire((sharing, i), p)
        return self

    def square_op(self, sharing, p):
        # two lin ops since the input is used twice
        self.lin_op(sharing, p)
        self.lin_op(sharing, p)

    def security(self, sharing):
        """Probability that all the shares of a given sharing are probed."""
        distr = self.distr()
        idxes = set(self.wire_idx((sharing, i)) for i in range(self.n_shares))
        offset = sum(2 ** i for i in idxes)
        if self.n_wires() == self.n_shares:
            return distr[offset]
        else:
            return sum(
                distr[offset + sum(x)]
                for x in it.product(
                    *((0, 2 ** i) for i in range(self.n_wires()) if i not in idxes)
                )
            )
