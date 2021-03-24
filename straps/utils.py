
import concurrent.futures
import os
import sys

from joblib import Memory

# Reduce this if you don't have enough RAM for the PDT multiplication stage.
MAX_WORKERS=os.cpu_count()

PARALLEL=True
#PARALLEL=False
def pmap(f, it, max_workers=MAX_WORKERS):
    if PARALLEL:
        def inner():
            with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as executor:
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


cache_dir=os.getenv("SERPEERS_CACHE_DIR")
if cache_dir:
    os.makedirs(cache_dir, exist_ok=True)
# If cache_dir is None, Memory acts as a transparent wrapper.
ldt_cache = Memory(cache_dir, verbose=0)
