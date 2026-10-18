function y = decimate(x, r)
%DECIMATE Decimate by an integer factor with an anti-aliasing prefilter.
%   Y = DECIMATE(X,R) lowpass-filters X and keeps every R-th sample.
%
%   Uses a windowed-sinc FIR (fir1, cutoff 1/R) applied zero-phase
%   (filtfilt), then decimation.  MATLAB defaults to an 8th-order Chebyshev
%   design; the passband behaviour is equivalent but exact samples differ.
    arguments
        x
        r double
    end
    x = reshape(x, numel(x), 1);
    nfilt = 2 * floor(5 * r) + 1;
    h = fir1(nfilt - 1, 1 / r);
    if numel(x) > 3 * (numel(h) - 1)
        % zero-phase filtering (matches MATLAB's filtfilt-based decimation)
        xf = filtfilt(h, 1, x);
    else
        % filtfilt needs a signal longer than 3*(nfilt-1); fall back
        xf = filter(h, 1, x);
    end
    y = xf(1:r:end);
end
