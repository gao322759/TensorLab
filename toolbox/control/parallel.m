function sys = parallel(sys1, sys2)
%PARALLEL Parallel connection of two SISO TF models: SYS = SYS1 + SYS2.
    arguments
        sys1
        sys2
    end
    n1 = sys1.Numerator;
    d1 = sys1.Denominator;
    n2 = sys2.Numerator;
    d2 = sys2.Denominator;
    den = conv(d1, d2);
    % numerator = n1*d2 + n2*d1 (zero padded to a common length)
    a = conv(n1, d2);
    b = conv(n2, d1);
    la = numel(a);
    lb = numel(b);
    L = la;
    if lb > L
        L = lb;
    end
    aa = zeros(1, L);
    bb = zeros(1, L);
    oa = L - la;
    ob = L - lb;
    for k = 1:la
        aa(oa + k) = a(k);
    end
    for k = 1:lb
        bb(ob + k) = b(k);
    end
    sys = tf(aa + bb, den);
end
