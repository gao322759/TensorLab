function w = rectwin(n)
%RECTWIN Rectangular window.
%   W = RECTWIN(N) returns an N-point rectangular window (all ones).
    arguments
        n
    end
    w = ones(n, 1);
end
