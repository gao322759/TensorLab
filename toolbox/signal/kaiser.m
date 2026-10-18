function w = kaiser(n, beta)
%KAISER Kaiser window.
%   W = KAISER(N,BETA) returns an N-point Kaiser window.
%
%   w(m) = I0(beta*sqrt(1-((m-a)/a)^2)) / I0(beta),  a = (N-1)/2,
%   with I0 the zeroth-order modified Bessel function of the first kind,
%   evaluated by its power series (Abramowitz & Stegun 9.6.12).
    arguments
        n double
        beta double = 0.5
    end
    if n <= 1
        w = ones(n, 1);
        return
    end
    m = (0:n-1)';
    a = (n - 1) / 2;
    r = (m - a) / a;
    z = beta * sqrt(1 - r.^2);

    s = ones(size(z));
    term = ones(size(z));
    for k = 1:60
        term = term .* (z / (2 * k)).^2;
        s = s + term;
    end
    t0 = beta;
    s0 = 1;
    term0 = 1;
    for k = 1:60
        term0 = term0 * (t0 / (2 * k))^2;
        s0 = s0 + term0;
    end
    w = s / s0;
end
