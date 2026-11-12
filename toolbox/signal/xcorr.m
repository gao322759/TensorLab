function r = xcorr(x, y)
%XCORR Cross-correlation (subset of MATLAB Signal Processing Toolbox).
%   R = XCORR(X)  returns the autocorrelation of X (2*N-1 lags).
%   R = XCORR(X,Y) returns the cross-correlation of X and Y; the shorter
%   vector is zero padded so the result has 2*max(M,N)-1 elements (MATLAB).
%
%   Algorithm: direct convolution with the conjugated, reversed operand
%   (Oppenheim & Schafer, Discrete-Time Signal Processing, ch. 2).
    arguments
        x
        y = []
    end
    rowIn = 0;
    if size(x, 1) == 1 && numel(x) > 1
        rowIn = 1;
    end
    x = reshape(x, numel(x), 1);
    if isempty(y)
        r = conv(x, flipud(conj(x)));
    else
        y = reshape(y, numel(y), 1);
        nx = numel(x);
        ny = numel(y);
        n = nx;
        if ny > n
            n = ny;
        end
        if nx < n
            x = [x; zeros(n - nx, 1)];
        end
        if ny < n
            y = [y; zeros(n - ny, 1)];
        end
        r = conv(x, flipud(conj(y)));
    end
    if rowIn
        r = reshape(r, 1, numel(r));
    end
end
