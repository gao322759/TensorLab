function x = chi2inv(p, nu)
%CHI2INV Inverse of the chi-square CDF (bisection on CHI2CDF).
%   X = CHI2INV(P,NU) returns x such that chi2cdf(x,NU) = P.
%   Subset: P must lie in (0,1); NU > 0 (non-integer allowed).
    arguments
        p double
        nu double
    end
    lo = 0;
    hi = 1;
    tries = 0;
    while chi2cdf(hi, nu) < p && tries < 60
        hi = hi * 2;
        tries = tries + 1;
    end
    for k = 1:200
        mid = (lo + hi) / 2;
        if chi2cdf(mid, nu) < p
            lo = mid;
        else
            hi = mid;
        end
    end
    x = (lo + hi) / 2;
end
