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

from setuptools import setup
from setuptools_rust import Binding, RustExtension

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="straps",
    version="0.1.3",
    author="Gaëtan Cassiers",
    author_email="gaetan.cassiers@uclouvain.be",
    description="Statistical Testing of RAndom Probing Security",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/cassiersg/straps",
    project_urls={
        "Bug Tracker": "https://github.com/cassiersg/straps/issues",
    },
    classifiers=[
        "Programming Language :: Python :: 3",
    ],
    rust_extensions=[
        RustExtension("straps._straps_ext", binding=Binding.PyO3, debug=False)
    ],
    packages=["straps"],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
    python_requires=">=3.10",
    install_requires=["joblib~=0.17", "matplotlib~=3.1", "numpy~=1.16", "tqdm~=4.51"],
)
