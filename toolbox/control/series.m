function sys = series(sys1, sys2)
%SERIES Series connection of two SISO TF models: SYS = SYS1 * SYS2.
    arguments
        sys1
        sys2
    end
    n1 = sys1.Numerator;
    d1 = sys1.Denominator;
    n2 = sys2.Numerator;
    d2 = sys2.Denominator;
    num = conv(n1, n2);
    den = conv(d1, d2);
    sys = tf(num, den);
end
