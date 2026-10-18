function r = xcorr(x, y)
%XCORR Cross-correlation (subset of MATLAB Signal Processing Toolbox).
%   R = XCORR(X)  returns the autocorrelation of X (2*N-1 lags).
%   R = XCORR(X,Y) returns the cross-correlation of X and Y.
%
%   Algorithm: direct convolution with the conjugated, reversed operand
%   (Oppenheim & Schafer, Discrete-Time Signal Processing, ch. 2).
    arguments
        x
        y = []
    end
    x = reshape(x, numel(x), 1);
    if isempty(y)
        r = conv(x, flipud(conj(x)));
    else
        y = reshape(y, numel(y), 1);
        r = conv(x, flipud(conj(y)));
    end
end
