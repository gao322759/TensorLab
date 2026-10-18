function [p, f] = pspectrum(x, fs)
%PSPECTRUM One-sided power spectrum of a real signal.
%   [P,F] = PSPECTrum(X,FS) returns the power spectrum of X sampled at FS
%   (default 1) and the frequency vector F. Subset of the MATLAB function:
%   rectangular window and 'power' scaling only.
    arguments
        x
        fs double = 1
    end
    x = x(:)';
    N = numel(x);
    X = fft(x);
    half = floor(N / 2) + 1;
    p = zeros(1, half);
    for k = 1:half
        p(k) = abs(X(k)) ^ 2 / (N * N);
    end
    for k = 2:half - 1
        p(k) = 2 * p(k);
    end
    if N / 2 ~= floor(N / 2)
        p(half) = 2 * p(half);
    end
    f = zeros(1, half);
    for k = 1:half
        f(k) = (k - 1) * fs / N;
    end
end
