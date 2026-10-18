function [z, p, k] = tf2zp(num, den)
%TF2ZP Transfer function to zero-pole-gain form.
%   [Z,P,K] = TF2ZP(NUM,DEN) with Z = roots(NUM), P = roots(DEN),
%   K = NUM(1)/DEN(1). Leading zero coefficients are stripped first, so a
%   strictly proper numerator (e.g. [0 1]) does not create a zero at 0.
    arguments
        num
        den
    end
    n2 = num;
    while numel(n2) > 1 && n2(1) == 0
        n2 = n2(2:end);
    end
    d2 = den;
    while numel(d2) > 1 && d2(1) == 0
        d2 = d2(2:end);
    end
    if numel(n2) <= 1
        z = [];
    else
        z = roots(n2);
    end
    if numel(d2) <= 1
        p = [];
    else
        p = roots(d2);
    end
    a1 = n2;
    if numel(n2) > 1
        a1 = n2(1);
    end
    b1 = d2;
    if numel(d2) > 1
        b1 = d2(1);
    end
    k = a1 / b1;
end
