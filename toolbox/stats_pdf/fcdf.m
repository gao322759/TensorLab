function p = fcdf(x, d1, d2)
%FCDF F cumulative distribution function.
%   P = FCDF(X,D1,D2) returns the CDF of the F distribution with D1
%   numerator and D2 denominator degrees of freedom.
    arguments
        x
        d1
        d2
    end
    if numel(x) == 1
        p = fcdf_one(x(1), d1, d2);
    else
        p = zeros(1, numel(x));
        for i = 1:numel(x)
            p(i) = fcdf_one(x(i), d1, d2);
        end
    end
end

function val = fcdf_one(v, d1, d2)
    val = 0;
    if v > 0
        z = d1 * v / (d1 * v + d2);
        a = d1 / 2;
        b = d2 / 2;
        bt = exp(gammaln(a + b) - gammaln(a) - gammaln(b) + a * log(z) + b * log(1 - z));
        if z < (a + 1) / (a + b + 2)
            val = bt * tl_betacf(z, a, b) / a;
        else
            val = 1 - bt * tl_betacf(1 - z, b, a) / b;
        end
    end
end

function y = tl_betacf(x, a, b)
%TL_BETACF Continued fraction for the incomplete beta function.
%   Local helper (Lentz's algorithm; Numerical Recipes, 3rd ed., ch. 6.4).
    tiny = 1e-300;
    qab = a + b;
    qap = a + 1;
    qam = a - 1;
    c = 1;
    d = 1 - qab * x / qap;
    if abs(d) < tiny
        d = tiny;
    end
    d = 1 / d;
    h = d;
    for m = 1:200
        m2 = 2 * m;
        aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1 + aa * d;
        if abs(d) < tiny
            d = tiny;
        end
        c = 1 + aa / c;
        if abs(c) < tiny
            c = tiny;
        end
        d = 1 / d;
        h = h * d * c;
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1 + aa * d;
        if abs(d) < tiny
            d = tiny;
        end
        c = 1 + aa / c;
        if abs(c) < tiny
            c = tiny;
        end
        d = 1 / d;
        del = d * c;
        h = h * del;
        if abs(del - 1) < 1e-12
            break
        end
    end
    y = h;
end
