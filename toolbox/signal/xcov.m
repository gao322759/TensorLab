function c = xcov(x, y)
%XCOV Cross-covariance (subset of MATLAB Signal Processing Toolbox).
%   C = XCOV(X)   returns the autocovariance of X (mean removed).
%   C = XCOV(X,Y) returns the cross-covariance of X and Y.
    arguments
        x
        y = []
    end
    if isempty(y)
        c = xcorr(x - mean(x));
    else
        c = xcorr(x - mean(x), y - mean(y));
    end
end
