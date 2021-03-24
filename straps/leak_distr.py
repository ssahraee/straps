
from ._straps_ext import PyLeakageDistribution

class LeakageDistribution:
    """Proxy adapter for PyLeakageDistribution.

    The main work is wire name adaptation: we support arbitrary types as wire
    names, while PyLeakageDistribution only supports strings.
    """
    def __init__(self, wires, distr=None):
        self.inner = PyLeakageDistribution(self.w2ss(wires))

    @staticmethod
    def w2s(wire):
        return '{}_{}'.format(*wire)

    @staticmethod
    def w2ss(wires):
        return ['{}_{}'.format(*w) for w in wires]

    def apply_op(self, inputs, outputs, ldt):
        self.inner = self.inner.apply_op(self.w2ss(inputs), self.w2ss(outputs), ldt)
        return self

    def leak_wire(self, var, p):
        self.inner = self.inner.leak_wire(self.w2s(var), p)
        return self

    def split_wire(self, src, dest1, dest2):
        self.inner = self.inner.split_wire(self.w2s(src), self.w2s(dest1), self.w2s(dest2))
        return self

    def distr(self):
        return self.inner.distr()

    def wire_idx(self, wire):
        return self.inner.wire_idx(self.w2s(wire))

    def n_wires(self):
        return len(self.inner.wires())

