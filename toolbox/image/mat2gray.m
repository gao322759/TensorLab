function I = mat2gray(A, limits)
%MAT2GRAY Convert a matrix to a normalized intensity image.
%   I = MAT2GRAY(A) linearly maps A to the range [0,1].
%   I = MAT2GRAY(A,[AMIN AMAX]) uses explicit limits; values outside the
%   limits are clipped to 0 and 1 (MATLAB semantics).  If AMIN == AMAX the
%   result is all ones.
    arguments
        A
        limits = []
    end
    if isempty(A)
        error('mat2gray:emptyInput', 'A must be non-empty');
    end
    if isempty(limits)
        lo = min(min(A));
        hi = max(max(A));
    else
        lo = limits(1);
        hi = limits(2);
    end
    if hi == lo
        I = ones(size(A));
        return
    end
    I = (A - lo) ./ (hi - lo);
    for i = 1:numel(I)
        if I(i) < 0
            I(i) = 0;
        elseif I(i) > 1
            I(i) = 1;
        end
    end
end
