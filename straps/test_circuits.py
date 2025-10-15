from straps import pdt_sampling
from straps import simple_circuits
import test_snih_4shares

d = 8
n_s_max = 10**3
suff_thresh = 10**3
circ_name = test_snih_4shares.snih_4_shares
use_copy = 1
print(pdt_sampling.serialized_cnt_pdt_raw(circ_name, d, n_s_max, suff_thresh, use_copy))

