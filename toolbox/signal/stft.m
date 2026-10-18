function [S, F, T] = stft(x, fs, win, noverlap, nfft)
%STFT Short-time Fourier transform (subset).
%   [S,F,T] = STFT(X,FS) uses a 128-point Hann window, 75% overlap and a
%   128-point FFT.  [S,F,T] = STFT(X,FS,WIN,NOVERLAP,NFFT) overrides them.
%   S is the one-sided spectrum (NFFT/2+1 by number of frames), F the
%   frequencies (Hz), T the frame-centre times (s).
%
%   Reference: Allen & Rabiner, Proc. IEEE 65(11), 1977.
    arguments
        x
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
    if numel(x) > 1
        x = x(:);
    end
    N = numel(x);
    w = hann(win);
    hop = win - noverlap;
    if hop < 1
        hop = 1;
    end
    nframes = floor((N - noverlap) / hop);
    if nframes < 1
        nframes = 1;
    end
    nbins = floor(nfft / 2) + 1;
    S = zeros(nbins, nframes);
    for k = 1:nframes
        i0 = (k - 1) * hop + 1;
        seg = zeros(win, 1);
        for j = 1:win
            idx = i0 + j - 1;
            if idx <= N
                seg(j) = x(idx);
            end
        end
        Y = fft(seg .* w, nfft);
        for m = 1:nbins
            S(m, k) = Y(m);
        end
    end
    T = (((0:nframes - 1) * hop) + win / 2) / fs;
    F = (0:nbins - 1)' * fs / nfft;
end
