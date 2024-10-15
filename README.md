# STRAPS

*Statistical Testing of RAndom Probing Security*

This tool is described in the [paper](https://epring.iacr.org/2021/880)
*Towards Tight Random Probing Security*.

## Install

STRAPS is distributed as a python package (with compiled native code in it).

Dependencies:

* `python >= 3.10` (for older python, see version 0.1.2)
* `pip`

(On Ubuntu: `apt install python3 python3-pip`. On Windows, install from <https://python.org>.)


Install command
```
pip install straps
```
or (install for local user only):
```
pip install --user straps
```

We do not currently build for Mac OS (working CI configuration contribution is
welcome), but you can build it for yourself (see below).

If the installation fails after "Building wheel for straps [...]", it is
probably due to the use of an old version of pip. If updating it is not
possible, you may also do it in python virtual environment:
```
python3 -m venv ve_straps
source ve_straps/bin/activate
python3 -m pip install -u pip
python3 -m pip install straps
```

## Usage

### Simple demo

```
python -m straps.secfig isw
```

Run
```
python -m straps.secfig --help
```
to see all options.

Running `python -m straps.paper_plots` generates all figures of the paper (this
might take dozens of hours on a beefy machine).

### Cache

If the environment variable `STRAPS_CACHE_DIR` is defined, it will be used as
the path for a cache directory. This cache stores PDT computation resuts across
executions, and also during one execution.
It is therefore **strongly recommended** to set this variable, as it might lead
to large runtime reductions, even on a single run.

### Custom composition

```python
from straps import eval_circs, sh_pdt, pdt_sampling, secfig

def eval_x_cube(p, pdts, d):
    """Composition to compute ISW-mul(x, x**2) (without refreshing)."""
    # Create the Shared PD with one output sharing
    x = sh_pdt.ShPd(['out'], d)
    # We build the circuit from the output: we start from the output sharing,
    # create the gadget that generates it, then work backwards until we reach
    # the intput.
    # ISW multiplication
    x.op('out', ['t0', 't1'], pdts['ISW'])
    x.op('t0', ['t0'], pdts['square'])
    x.split_sharing('in', 't0', 't1')
    return x.security('in')

## Then, either run
# Set the parameters:
k = "ub" # ub (upper bound) or lb (statistical-only lower bound)
e = 1e-6 # statistical confidence level
d = 3 # number of shares
n_s_max = 10**5 # N_max
suff_thresh = 100 # N_t
p = 1e-2 # parameter of the random probing model
pdts = {
    circ: pdt_sampling.gpdt(circ, d, k, e, n_s_max, suff_thresh, True, False).instantiate(p)
    for circ in ["ISW", "square"]
    }
# Get the security level:
security_level = eval_x_cube(p, pdts, d)

## Or, if you want to integrate with provided utils:
# Put in base_circuits your custom function and the list of gadgets you use
eval_circs.base_circuits["custom_cube_implem"] = (eval_x_cube, lambda **kwargs: ['ISW', 'square'])
# Put in specialized_circuits a display name, and the name of your base_circuits entry
# (and a dict of optional parameter to your function).
eval_circs.specialized_circuits["custom_cube"] = ("ISW Cube w/o refresh", "custom_cube_implem", {})
# Then, you can use our top-level functions, e.g.
import numpy as np
from matplotlib import pyplot as plt
ds = [1, 2, 3] # number of shares
ps = np.logspace(-4, 0, 50) # parameter of the random probing model
e = 1e-6 # statistical confidence level
n_s_max = 10**5 # N_max
suff_thresh = 100 # N_t
secfig.plot_fig(**secfig.data_fig("custom_cube", ds, e, ps, n_s_max, suff_thresh))
plt.show()
```

See `straps/eval_circs.py` for more examples (such as the AES S-box).

### Custom gadget

Your can also design your own gadget.
```python
from straps import circuit_model

# Define the gadget.
def custom_gadget(d):
    """Custom gadget with d shares."""
    if d != 2:
        raise ValueError("This gadget works only with 2 shares.")
    c = circuit_model.Circuit(d)
    # two input sharings: (in00, in01) and (in10, in11)
    in00 = c.var("in00", kind="input", port=(0, 0))
    in01 = c.var("in01", kind="input", port=(0, 1))
    in10 = c.var("in10", kind="input", port=(1, 0))
    in11 = c.var("in11", kind="input", port=(1, 1))
    # one output sharing (out0, out1)
    out0 = c.var("out0", kind="output", port=(0, 0))
    out1 = c.var("out1", kind="output", port=(0, 1))
    # a fresh random
    r = c.var("r", kind="random")
    # intermediate variables
    w = c.var("w")
    x = c.var("x")
    y = c.var("y")
    # circuit gates
    c.l_sum(w, (in00, r)) # XOR gate: x = in00 XOR r
    c.l_sum(x, (w, in01))
    c.l_sum(y, (in10, in11)) # NB: leaks at first-order.
    c.l_prod(out0, (y, x)) # AND gate: out0 = x AND y
    c.l_prod(out1, (y, r))
    return c

# Integrate the gadget in the list of available gadgets:
from straps import simple_circuits
simple_circuits.all_circs["my_custom_gadget"] = custom_gadget

# Then you can use "my_custom_gadget" in any custom composition (see Custom
# composition section). E.g.
from straps import sh_pdt, eval_circs
def eval_custom_gadget(p, pdts, d, sec_input="in0"):
    x = sh_pdt.ShPd(['out'], d)
    x.op('out', ['in0', 'in1'], pdts['my_custom_gadget'])
    return x.security(sec_input)

eval_circs.base_circuits["custom_gadget"] = (
        eval_custom_gadget, lambda **kwargs: ['my_custom_gadget']
)
eval_circs.specialized_circuits["custom_gadget_in0"] = ("Custom Gadget in 0", "custom_gadget", {'sec_input': 'in0'})
eval_circs.specialized_circuits["custom_gadget_in1"] = ("Custom Gadget in 1", "custom_gadget", {'sec_input': 'in1'})
# You can then evaluate the security with straps.secfig (see Custom composition section).
```

## Build

If you want to build STRAPS yourself, you will need the following for all platforms:

* A stable rust compiler with cargo (install e.g. from <https://rustup.rs>)
* Python (>= 3.6)
* The boost library:
    * On Ubuntu (20.04):
    ```
    apt install libboost-all-dev
    ```
    * On RHEL/CentOS:
    ```
    yum install boost-devel
    ```
    * On Windows (using [Chocolatey](https://chocolatey.org)
    ```
    choco install boost-msvc-14.2
    ```
    (Assuming Visual Studio 2019)
* A C++ compiler
    * On Ubuntu (20.04):
    ```
    apt install gcc g++
    ```
    * On RHEL/CentOS:
    ```
    yum install gcc gcc-g++
    ```
    * On Windows install Visual Studio 2019 with C++ extensions.


Then, run
```
python setup.py develop
```
to install STRAPS in development mode.
For Windows, you need to the the environment variable
`CXXFLAGS=-I C:/Local/boost_1_74_0` (adjust according to your boost version).

## License

STRAPS is licensed under the GNU AGPL, version 3 or later.
See [COPYING](COPYING) for details.

