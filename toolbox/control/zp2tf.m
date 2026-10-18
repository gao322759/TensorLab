function [num, den] = zp2tf(z, p, k)
%ZP2TF Zero-pole-gain to transfer function.
%   [NUM,DEN] = ZP2TF(Z,P,K) with DEN = poly(P) and NUM = K*poly(Z).
%   Polynomials are built by repeated convolution (highest power first).
    arguments
        z
        p
        k double = 1
    end
    den = 1;
    if isempty(p)
        den = 1;
    elseif numel(p) == 1
        den = [1, -p];
    else
        for i = 1:numel(p)
            den = conv(den, [1, -p(i)]);
        end
    end

    num = 1;
    if isempty(z)
        num = 1;
    elseif numel(z) == 1
        num = [1, -z];
    else
        for i = 1:numel(z)
            num = conv(num, [1, -z(i)]);
        end
    end
    num = k * num;
end
