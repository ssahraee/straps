// STRAPS - Statistical Testing of RAndom Probing Security
// Copyright (C) 2021 UCLouvain
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#include <boost/math/special_functions.hpp>
#include <boost/exception/diagnostic_information.hpp>
#include <iostream>

namespace boost
{
#ifdef BOOST_NO_EXCEPTIONS
void throw_exception( std::exception const & e ){
    throw e;
};
#endif
}// namespace boost

extern "C" int ibeta_inv(double a, double b, double p, double *res, double *py) {
    try {
    *res = boost::math::ibeta_inv(a, b, p, py);
    } catch (const boost::exception& e)
        {
            std::string diag = diagnostic_information(e);
            std::cout << "Boost exception" << diag;
            return 1;
        }
        return 0;
}
extern "C" int ibetac_inv(double a, double b, double q, double *res, double *py) {
    try {
    *res = boost::math::ibetac_inv(a, b, q, py);
    } catch (const boost::exception& e)
        {
            std::string diag = diagnostic_information(e);
            std::cout << "Boost exception" << diag;
            return 1;
        }
        return 0;
}
