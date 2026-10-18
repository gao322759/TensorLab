function h = evalfr(sys, s)
%EVALFR Evaluate a transfer function at a complex point s.
%   H = EVALFR(SYS,S) returns NUM(S)/DEN(S) for the SISO TF model SYS.
    arguments
        sys
        s
    end
    num = sys.Numerator;
    den = sys.Denominator;
    h = polyval(num, s) / polyval(den, s);
end
