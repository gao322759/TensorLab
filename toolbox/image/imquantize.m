function J = imquantize(I, levels)
%IMQUANTIZE Quantize an image to discrete levels.
%   J = IMQUANTIZE(I,LEVELS) replaces each value with its bin index
%   (1 .. numel(LEVELS)+1); LEVELS must be monotonically increasing.
    arguments
        I
        levels
    end
    sz = size(I);
    J = ones(sz(1), sz(2));
    nl = numel(levels);
    for i = 1:sz(1)
        for j = 1:sz(2)
            v = I(i, j);
            k = 1;
            while k <= nl && v > levels(k)
                k = k + 1;
            end
            J(i, j) = k;
        end
    end
end
