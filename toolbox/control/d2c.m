function sysc = d2c(sys, Ts, method)
%D2C Discrete-to-continuous conversion (bilinear/Tustin).
%   SYSC = D2C(SYS,TS) converts a discrete transfer function sampled at TS
%   to continuous time using z = (1 + s*Ts/2)/(1 - s*Ts/2).
%   MATLAB reads TS from the model; this kernel build requires it explicitly.
    arguments
        sys
        Ts double
        method = 'tustin'
    end
    num = sys.Numerator;
    den = sys.Denominator;
    d = max(numel(num), numel(den)) - 1;
    nc = inv_bilinear(num, Ts, d);
    dc = inv_bilinear(den, Ts, d);
    lead = dc(1);
    dc = dc / lead;
    nc = nc / lead;
    sysc = tf(nc, dc);
end

function q = inv_bilinear(p, Ts, d)
    len = numel(p);
    dp = len - 1;
    q = 0;
    for i = 1:len
        k = dp - (i - 1);
        term = p(i);
        c1 = 1;
        for m = 1:k
            c1 = conv(c1, [Ts / 2, 1]);
        end
        c2 = 1;
        for m = 1:(d - k)
            c2 = conv(c2, [-Ts / 2, 1]);
        end
        q = q + term * conv(c1, c2);
    end
end
