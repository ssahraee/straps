from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="straps",
    version="0.1.0",
    rust_extensions=[
		RustExtension("straps._straps_ext", binding=Binding.PyO3, features=["pyo3/abi3"], py_limited_api=True,debug=False)
		],
    packages=["straps"],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
	python_requires='~=3.5',
	install_requires=["joblib~=0.17", "matplotlib~=3.1", "numpy~=1.16", "tqdm~=4.51"],
)
