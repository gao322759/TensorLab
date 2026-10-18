function [n, Wn, beta, ftype] = kaiserord(f, a, dev, fs)
%KAISERORD Kaiser window filter order estimate.
%   [N,WN,BETA,FTYPE] = KAISERORD(F,A,DEV,FS) estimates the FIR order needed
%   to meet the band specification F/A/DEV sampled at FS.
%
%   Formulas from Oppenheim & Schafer, Discrete-Time Signal Processing, 3rd
%   ed., ch. 7 (Kaiser's empirical order rule).
    arguments
        f
        a
        dev
        fs double = 2
    end
    f = f(:)' / (fs / 2);
    a = a(:)';
    dev = dev(:)';
    if numel(f) ~= 2 * numel(a)
        error('kaiserord:bands', 'f must contain 2*numel(a) band edges');
    end
    d = min(dev);
    A = -20 * log10(d);
    if A > 50
        beta = 0.1102 * (A - 8.7);
    elseif A > 21
        beta = 0.5842 * (A - 21) ^ 0.4 + 0.07886 * (A - 21);
    else
        beta = 0;
    end
    nb = numel(a);
    dw = 1;
    first = 1;
    for k = 1:nb - 1
        gap = f(2 * k + 1) - f(2 * k);
        if first == 1 || gap < dw
            dw = gap;
            first = 0;
        end
    end
    if dw <= 0
        error('kaiserord:bands', 'band edges must be strictly increasing');
    end
    n = ceil((A - 8) / (2.285 * dw * pi));
    if nb - 1 == 1
        Wn = (f(2) + f(3)) / 2;
    else
        Wn = zeros(1, nb - 1);
        for k = 1:nb - 1
            Wn(k) = (f(2 * k) + f(2 * k + 1)) / 2;
        end
    end
    if nb == 2
        if a(1) > a(2)
            ftype = 'low';
        else
            ftype = 'high';
        end
    else
        if a(1) > a(2)
            ftype = 'bandpass';
        else
            ftype = 'bandstop';
        end
    end
end
