function I = mat2gray(A, limits)
%MAT2GRAY Convert a matrix to a normalized intensity image.
%   I = MAT2GRAY(A) linearly maps A to the range [0,1].
%   I = MAT2GRAY(A,[AMIN AMAX]) uses explicit limits.
    arguments
        A
        limits = []
    end
    if isempty(limits)
        lo = min(A(:));
        hi = max(A(:));
    else
        lo = limits(1);
        hi = limits(2);
    end
    if hi == lo
        I = zeros(size(A));
    else
        I = (A - lo) ./ (hi - lo);
    end
end
