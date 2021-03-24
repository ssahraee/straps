#include <boost/math/special_functions.hpp>
#include <boost/exception/diagnostic_information.hpp>
#include <iostream>

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
