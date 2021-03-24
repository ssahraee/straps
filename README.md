# STRAPS

*Statistical Testing of RAndom Probing Security*

This tool is described in the [paper](https://epring.iacr.org/2021/TODO)
*Towards Tight Random Probing Security*.

## Install

STRAPS is distributed as a python package (with compiled native code in it).

Dependencies:

* `python >= 3.6`
* `pip`

Install command
```
pip install straps
```

We do not currently build for Mac OS (working CI configuration contribution is
welcome), but you can build it for yourself (see below).

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
from straps import eval_circs, sh_ldt, ldt_sampling, secfig

def eval_x_cube(p, pdts, d):
    """Composition to compute ISW-mul(x, x**2) (without refreshing)."""
    # Create the Shared PD with one output sharing
    x = sh_ldt.ShLd(['out'], d)
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
pdts = {
    circ: ldt_sampling.gpdt(circ, d, k, e, n_s_max, suff_thresh, True, False).instantiate(p)
    for circ in ["ISW", "square]
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
secfig.plot_fig(**secfig.data_fig("custom_cube", ds, err, ps, n_s_max, suff_thresh))
plt.show()
```

See `straps/eval_circs.py` for more examples (such as the AES S-box).

### Custom gadget

Your can also design your own gadget.
```python
from straps import circuit_model

def custom_gadget(d):
    """Useless custom gadget with d shares."""
    if d != 2:
        raise ValueError("This gadget works only with 2 shares...")
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
python setup.py devlop
```
to install STRAPS in development mode.
For Windows, you need to the the environment variable
`CXXFLAGS=-I C:/Local/boost_1_74_0` (adjust according to your boost version).

## License

STRAPS is licensed under the GNU AGPL, version 3 or later.
See [COPYING](COPYING) for details.

