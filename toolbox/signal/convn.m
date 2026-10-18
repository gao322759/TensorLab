function c = convn(a, b, shape)
%CONVN 2-D convolution (N-D subset).
%   C = CONVN(A,B) returns the full 2-D convolution of A and B.
%   C = CONVN(A,B,'same') returns the central part, same size as A.
%   C = CONVN(A,B,'valid') returns the part computed without padding.
%
%   Direct summation (textbook definition). Higher-dimensional inputs are
%   rejected; this kernel build has no reliable RESHAPE inside functions.
    arguments
        a
        b
        shape = 'full'
    end
    if size(a, 3) > 1 || size(b, 3) > 1
        error('convn:dims', 'convn currently supports 1-D and 2-D inputs');
    end
    ra = size(a, 1);
    ca = size(a, 2);
    rb = size(b, 1);
    cb = size(b, 2);
    ro = ra + rb - 1;
    co = ca + cb - 1;

    full = zeros(ro, co);
    for i = 1:ra
        for j = 1:ca
            v = a(i, j);
            if v ~= 0
                for k = 1:rb
                    for l = 1:cb
                        w = b(k, l);
                        if w ~= 0
                            full(i + k - 1, j + l - 1) = full(i + k - 1, j + l - 1) + v * w;
                        end
                    end
                end
            end
        end
    end

    if strcmp(shape, 'same')
        i0 = floor(rb / 2);
        j0 = floor(cb / 2);
        c = zeros(ra, ca);
        for i = 1:ra
            for j = 1:ca
                c(i, j) = full(i + i0, j + j0);
            end
        end
    elseif strcmp(shape, 'valid')
        rv = ro - 2 * (rb - 1);
        cv = co - 2 * (cb - 1);
        if rv < 1 || cv < 1
            error('convn:valid', 'valid convolution requires B no larger than A');
        end
        c = zeros(rv, cv);
        for i = 1:rv
            for j = 1:cv
                c(i, j) = full(i + rb - 1, j + cb - 1);
            end
        end
    else
        c = full;
    end
end
