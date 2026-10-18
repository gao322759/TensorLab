function sysd = c2d(sys, Ts, method)
%C2D Continuous-to-discrete conversion of a SISO transfer function.
%   SYSD = C2D(SYS,TS) discretizes SYS with a zero-order hold (default).
%   SYSD = C2D(SYS,TS,METHOD) supports 'zoh' and 'tustin'/'bilinear'.
%
%   ZOH uses the residue expansion (distinct, non-zero poles); Tustin uses
%   the bilinear substitution s = 2*(z-1)/(Ts*(z+1))
%   (Franklin, Powell & Workman, Digital Control of Dynamic Systems, 3rd ed.).
    arguments
        sys
        Ts double
        method = 'zoh'
    end
    num = sys.Numerator;
    den = sys.Denominator;
    if strcmp(method, 'tustin') || strcmp(method, 'bilinear')
        [nz, dz] = bilinear_sub(num, den, Ts);
        sysd = tf(nz, dz);
        return
    end
    if ~strcmp(method, 'zoh')
        error('c2d:method', 'supported methods: zoh, tustin');
    end

    dn = numel(den) - 1;
    nn = numel(num) - 1;
    if nn >= dn
        error('c2d:proper', 'zoh conversion requires a strictly proper transfer function');
    end
    p = roots(den);
    if numel(p) < 1
        sysd = tf(num, den);
        return
    end
    if numel(p) == 1
        a = p;
        if numel(p) > 1
            a = p(1);
        end
        ad = exp(a * Ts);
        r = polyval(num, a);
        numz = zeros(1, 2);
        numz(2) = r * (ad - 1) / a;
        denz = zeros(1, 2);
        denz(1) = 1;
        denz(2) = -ad;
        sysd = tf(numz, denz);
        return
    end
    % Distinct, non-zero poles only (fallback: Tustin).
    for i = 1:numel(p)
        if abs(p(i)) < 1e-9
            [nz, dz] = bilinear_sub(num, den, Ts);
            sysd = tf(nz, dz);
            return
        end
        for j = i + 1:numel(p)
            if abs(p(i) - p(j)) < 1e-8
                [nz, dz] = bilinear_sub(num, den, Ts);
                sysd = tf(nz, dz);
                return
            end
        end
    end

    % Derivative of the denominator for the residues.
    dd = zeros(1, numel(den) - 1);
    for i = 1:numel(den) - 1
        dd(i) = den(i) * (numel(den) - i);
    end

    Dz = 1;
    A = zeros(1, numel(p));
    for i = 1:numel(p)
        A(i) = exp(p(i) * Ts);
        Dz = conv(Dz, [1, -A(i)]);
    end

    N = zeros(1, numel(Dz));
    for i = 1:numel(p)
        % P_i = product over j ~= i of (z - a_j)
        Pi = 1;
        for j = 1:numel(p)
            if j ~= i
                Pi = conv(Pi, [1, -A(j)]);
            end
        end
        ri = polyval(num, p(i)) / polyval(dd, p(i));
        ci = ri * (A(i) - 1) / p(i);
        for k = 1:numel(Pi)
            N(k) = N(k) + real(ci * Pi(k));
        end
    end

    % Normalize the leading denominator coefficient to 1.
    lead = Dz(1);
    nz = N / lead;
    dz = Dz / lead;
    sysd = tf(nz, dz);
end

function [nz, dz] = bilinear_sub(num, den, Ts)
    d = max(numel(num), numel(den)) - 1;
    nz = sub_poly(num, Ts, d);
    dz = sub_poly(den, Ts, d);
    lead = dz(1);
    dz = dz / lead;
    nz = nz / lead;
end

function q = sub_poly(p, Ts, d)
    len = numel(p);
    dp = len - 1;
    q = 0;
    for i = 1:len
        k = dp - (i - 1);
        term = p(i) * (2 / Ts) ^ k;
        c1 = 1;
        for m = 1:k
            c1 = conv(c1, [1, -1]);
        end
        c2 = 1;
        for m = 1:(d - k)
            c2 = conv(c2, [1, 1]);
        end
        q = q + term * conv(c1, c2);
    end
end
