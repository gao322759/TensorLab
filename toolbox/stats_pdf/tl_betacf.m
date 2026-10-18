function y = tl_betacf(x, a, b)
%TL_BETACF Continued fraction for the incomplete beta function.
%   Internal helper (Lentz's algorithm; Numerical Recipes, 3rd ed., ch. 6.4).
    arguments
        x
        a
        b
    end
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
