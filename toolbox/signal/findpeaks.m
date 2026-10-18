function [pks, locs] = findpeaks(x)
%FINDPEAKS Local maxima (subset of MATLAB Signal Processing Toolbox).
%   [PKS,LOCS] = FINDPEAKS(X) returns the local maxima of vector X and
%   their 1-based indices. Interior local maxima only (endpoints excluded),
%   matching MATLAB's default.
    arguments
        x
    end
    x = reshape(x, numel(x), 1);
    n = numel(x);
    pks = zeros(0, 1);
    locs = zeros(0, 1);
    if n >= 3
        c = x(2:n-1);
        locs = (2:n-1)';
        keep = (c > x(1:n-2)) & (c > x(3:n));
        locs = locs(keep);
        pks = x(locs);
    end
end
