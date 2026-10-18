function [x, t] = istft(S, fs, win, noverlap, nfft)
%ISTFT Inverse short-time Fourier transform (overlap-add).
%   [X,T] = ISTFT(S,FS) reconstructs a real signal from the one-sided
%   spectrum S produced by STFT with matching defaults.
%   [X,T] = ISTFT(S,FS,WIN,NOVERLAP,NFFT) overrides the analysis parameters.
    arguments
        S
        fs
        win double = 128
        noverlap double = []
        nfft double = []
    end
    if isempty(noverlap)
        noverlap = round(0.75 * win);
    end
    if isempty(nfft)
        nfft = win;
    end
    nbins = floor(nfft / 2) + 1;
    nframes = size(S, 2);
    hop = win - noverlap;
    if hop < 1
        hop = 1;
    end
    len = (nframes - 1) * hop + win;
    x = zeros(len, 1);
    wsum = zeros(len, 1);
    w = hann(win);
    for k = 1:nframes
        Y = zeros(nfft, 1);
        for m = 1:nbins
            Y(m) = S(m, k);
        end
        for m = 2:nbins - 1
            Y(nfft - m + 2) = conj(S(m, k));
        end
        seg = real(ifft(Y));
        i0 = (k - 1) * hop + 1;
        for j = 1:win
            idx = i0 + j - 1;
            x(idx) = x(idx) + seg(j) * w(j);
            wsum(idx) = wsum(idx) + w(j)^2;
        end
    end
    for i = 1:len
        if wsum(i) > 1e-12
            x(i) = x(i) / wsum(i);
        end
    end
    t = (0:len - 1)' / fs;
end
