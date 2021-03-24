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


from matplotlib import pyplot as plt

from . import secfig

def plot_subplot(fns, **kwargs):
    nx = len(fns)
    ny = len(fns[0])
    for i, fnsi in enumerate(fns):
        for j, fn in enumerate(fnsi):
            if not isinstance(fn, str):
                fn, supp_args = fn
            else:
                supp_args = dict()
            plt.subplot(nx, ny, ny*i+j+1)
            secfig.plot_fig(**secfig.data_fig(fn, **kwargs, **supp_args))

def plot_subplot_title(title, fns, **kwargs):
    plt.figure(title)
    plt.title(title)
    plot_subplot(fns, **kwargs)

def main():
    if False:
        plot_subplot_title('ISW',
                [ [ ('isw', {'use_copy': False, 'cum_tr': False}),
                        ('isw', {'use_copy': False, 'cum_tr': True}), ],
                    #[     ('isw', {'use_copy': True, 'cum_tr': False}),
                    #    ('isw', {'use_copy': True, 'cum_tr': True}), ],
                    ],
                ds=range(1,6),
                suff_thresh=100,
                n_s_max=10**6
                )

    # ok, to plot
    if False:
        plot_subplot_title('ISW', [['sh-add', 'isw'], ['isw_y', 'isw_xy']],
                )
    # ok, to plot
    if False:
        plot_subplot_title('ISW_cube', [['isw', 'cube_isw_noref'], ['cube_isw_simpleref', 'cube_isw_optref']])

    # old
    if False:
        plot_subplot_title('direct_vs_comp cube',
                [['cube_isw_noref', 'cube_isw_simpleref', 'cube_isw_optref'],
                    ['int_cube_isw_noref', 'int_cube_isw_simpleref', 'int_cube_isw_optref']],
                ds=range(1,5),
                n_s_max=10**7,
                suff_thresh=1000,
                )
    # ok, to plot
    if False:
        plot_subplot_title('direct_vs_comp cube',
                [['cube_isw_optref', 'int_cube_isw_optref']],
                ds=list(range(1, 6)),
                n_s_max=2*10**6,
                suff_thresh=1000,
                )
    # ok, to plot
    if True:
        plot_subplot_title('aes_sbox', [['isw', 'aes_sbox_noref'],
            ['aes_sbox_simpleref', 'aes_sbox']],
            n_s_max=10**8,
            ds=list(range(1,7))
            )

    # ok, to plot
    if False:
        plot_subplot_title('isw_ns2', [
            [
                ('isw', {'n_s_max': 10**3, 'suff_thresh': 1000}),
                ('isw', {'n_s_max': 10**5, 'suff_thresh': 1000}),
                ('isw', {'n_s_max': 10**7, 'suff_thresh': 1000}),
                ],
            ],
            ds=range(1,7),
        )
    # ok, to plot
    if False:
        plot_subplot_title('isw_ns2', [
            [
                ('isw', {'n_s_max': 10**7, 'suff_thresh': 10}),
                ('isw', {'n_s_max': 10**7, 'suff_thresh': 100}),
                ('isw', {'n_s_max': 10**7, 'suff_thresh': 1000}),
                ],
            ],
            ds=range(1,7),
        )

    # ongoing...
    if False:
        plot_subplot_title('isw_ns3', [ [ ('isw', {'n_s_max': 10**8, 'suff_thresh': 1000}), ], ],
            ds=[6],
        )
    plt.show()

if __name__ == '__main__':
    main()
