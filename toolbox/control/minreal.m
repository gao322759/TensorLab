function sys2 = minreal(sys, tol)
%MINREAL Minimal realization by pole-zero cancellation.
%   SYS2 = MINREAL(SYS) cancels approximately common zeros and poles of a
%   SISO transfer-function model.
%   SYS2 = MINREAL(SYS,TOL) uses an absolute tolerance scaled by (1+|p|).
    arguments
        sys
        tol double = 1e-4
    end
    num = sys.Numerator;
    den = sys.Denominator;
    z = roots(num);
    p = roots(den);
    k = num(1) / den(1);

    usedp = zeros(1, numel(p));
    keepz = [];
    for i = 1:numel(z)
        best = 0;
        bd = 1e300;
        for j = 1:numel(p)
            if usedp(j) == 0
                d = abs(z(i) - p(j));
                if d < bd
                    bd = d;
                    best = j;
                end
            end
        end
        if best > 0 && bd < tol * (1 + abs(p(best)))
            usedp(best) = 1;
        else
            keepz = [keepz; z(i)];
        end
    end
    keepp = [];
    for j = 1:numel(p)
        if usedp(j) == 0
            keepp = [keepp; p(j)];
        end
    end

    num2 = k;
    for i = 1:numel(keepz)
        num2 = conv(num2, [1, -keepz(i)]);
    end
    den2 = 1;
    for i = 1:numel(keepp)
        den2 = conv(den2, [1, -keepp(i)]);
    end
    sys2 = tf(num2, den2);
end
