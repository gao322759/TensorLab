function [A, B, C, D] = tf2ss(num, den)
%TF2SS Transfer function to state-space (controllable canonical form).
%   [A,B,C,D] = TF2SS(NUM,DEN) for a SISO transfer function NUM(s)/DEN(s).
%   The denominator is normalized so DEN(1) = 1.
%   Note: avoids RESHAPE (kernel stack-overflow pitfall).
    arguments
        num
        den
    end
    nd = numel(den);
    nn = numel(num);
    d = zeros(1, nd);
    if nd == 1
        d(1) = den;
    else
        for k = 1:nd
            d(k) = den(k);
        end
    end
    nv = zeros(1, nn);
    if nn == 1
        nv(1) = num;
    else
        for k = 1:nn
            nv(k) = num(k);
        end
    end

    if d(1) == 0
        error('tf2ss:den', 'denominator leading coefficient must be non-zero');
    end
    k0 = d(1);
    for k = 1:nd
        d(k) = d(k) / k0;
    end
    for k = 1:nn
        nv(k) = nv(k) / k0;
    end

    n = nd - 1;
    if n < 1
        error('tf2ss:order', 'denominator must have degree at least 1');
    end

    % pad the numerator to length n+1 (leading zeros)
    np = zeros(1, n + 1);
    off = n + 1 - nn;
    for k = 1:nn
        np(off + k) = nv(k);
    end

    A = zeros(n, n);
    if n == 1
        A(1, 1) = -d(2);
    else
        for j = 1:n
            A(1, j) = -d(j + 1);
        end
        for i = 2:n
            A(i, i - 1) = 1;
        end
    end
    B = zeros(n, 1);
    B(1) = 1;
    C = zeros(1, n);
    if n == 1
        C(1, 1) = np(2) - np(1) * d(2);
    else
        for j = 1:n
            C(j) = np(j + 1) - np(1) * d(j + 1);
        end
    end
    D = np(1);
end
