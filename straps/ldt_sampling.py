
import functools as ft

from ._straps_ext import PyCntSimSt

from . import simple_circuits
from . import utils

# We serialize to have simple way to store to disk for caching.
# The proper way would be to implement the proper serialization/deserialization
# on the rust objects, but this is currently not supported by pyo3.
@utils.ldt_cache.cache
def serialized_cnt_ldt_raw(circ_name, d, n_s_max, suff_thresh, use_copy):
    print('cnt_ldt', circ_name, 'd={} n_s_max={} suff_thresh={} use_copy: {}'.format(
        d, n_s_max, suff_thresh, use_copy
        ))
    pcg = simple_circuits.circ2pcg(circ_name, d)
    pcntsim = pcg.cnt_sim(use_copy)
    pcntsimst = utils.interruptible(pcntsim.run_sampling, n_s_max, suff_thresh)
    return {
            'ldt_sampling': pcntsimst.to_array(),
            'exhaustive': pcntsimst.exhaustive(),
            }

def cnt_ldt_raw(circ_name, d, n_s_max, suff_thresh, use_copy):
    res = serialized_cnt_ldt_raw(circ_name, d, n_s_max, suff_thresh, use_copy)
    res['pcntsim'] = PyCntSimSt(res['ldt_sampling'], res['exhaustive'])
    return res

# We don't bother using a disk cache for this, as it is reasonnably fast to
# compute, but an in-memory cache doesn't hurt.
@ft.lru_cache(maxsize=None)
def gpdt(circ_name, d, kind, err, n_s_max, suff_thresh, use_copy, cum_tr):
    """PDT marginalized by probe count."""
    l = cnt_ldt_raw(circ_name, d, n_s_max, suff_thresh, use_copy)
    if kind == 'est':
        res = l['pcntsim'].estimate()
    elif kind == 'ub':
        res = l['pcntsim'].ub(err, cum_tr)
    elif kind == 'lb':
        res = l['pcntsim'].lb(err, cum_tr)
    else:
        raise ValueError(repr(kind))
    return res

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description='Run sampling.')
    parser.add_argument('--circname', default='ISW')
    parser.add_argument('--nshares', default=2)
    parser.add_argument('--nsmax', default=10**3)
    parser.add_argument('--suffthresh', default=10**3)
    parser.add_argument('--usecopy', default=1)
    args = parser.parse_args()
    serialized_cnt_ldt_raw(args.circname, int(args.nshares), int(args.nsmax),
            int(args.suffthresh), bool(int(args.usecopy)))

