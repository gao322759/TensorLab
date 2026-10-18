function J = imnoise(I, type, a, b)
%IMNOISE Add noise to an intensity image.
%   J = IMNOISE(I,'gaussian',M,V)   additive Gaussian noise (default M=0, V=0.01).
%   J = IMNOISE(I,'salt & pepper',D) impulse noise of density D (default 0.05).
%   J = IMNOISE(I,'speckle',V)       multiplicative noise (default V=0.04).
    arguments
        I
        type = 'gaussian'
        a double = 0
        b double = 0.01
    end
    sz = size(I);
    if strcmpi(type, 'speckle')
        J = I + I .* randn(sz) * sqrt(b);
    elseif strcmpi(type, 'gaussian')
        J = I + a + sqrt(b) * randn(sz);
    else
        % salt & pepper
        d = 0.05;
        if a > 0
            d = a;
        end
        J = I;
        for i = 1:sz(1)
            for j = 1:sz(2)
                r = rand();
                if r < d / 2
                    J(i, j) = 0;
                end
                if r > 1 - d / 2
                    J(i, j) = 1;
                end
            end
        end
    end
    for i = 1:sz(1)
        for j = 1:sz(2)
            v = J(i, j);
            if v < 0
                v = 0;
            end
            if v > 1
                v = 1;
            end
            J(i, j) = v;
        end
    end
end
