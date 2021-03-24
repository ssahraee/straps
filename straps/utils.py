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


import concurrent.futures
import os
import sys

from joblib import Memory

# Reduce this if you don't have enough RAM for the PDT multiplication stage.
MAX_WORKERS = os.cpu_count()

PARALLEL = True
# PARALLEL=False
def pmap(f, it, max_workers=MAX_WORKERS):
    if PARALLEL:

        def inner():
            with concurrent.futures.ThreadPoolExecutor(
                max_workers=max_workers
            ) as executor:
                yield from executor.map(f, it)

        return inner()
    else:
        return map(f, it)


def interruptible(fn, *args, **kwargs):
    """Run fn in another thread. This enables to keep processing signals (hence
    KeyboardInterrupt) when fn is a long-running non-python function
    that releases the GIL.
    """
    executor = concurrent.futures.ThreadPoolExecutor(max_workers=1)
    future = executor.submit(fn, *args, **kwargs)
    try:
        return future.result()
    except:
        future.cancel()
        executor.shutdown(wait=True)
        raise


cache_dir = os.getenv("STRAPS_CACHE_DIR")
if cache_dir:
    os.makedirs(cache_dir, exist_ok=True)
# If cache_dir is None, Memory acts as a transparent wrapper.
pdt_cache = Memory(cache_dir, verbose=0)
