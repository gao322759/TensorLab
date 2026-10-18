function w = tukeywin(n, r)
%TUKEYWIN Tukey (tapered cosine) window.
%   W = TUKEYWIN(N)     returns an N-point Tukey window with r = 1/2.
%   W = TUKEYWIN(N,R)   R is the cosine fraction in [0,1]:
%                       R = 0 -> rectangular, R = 1 -> Hann.
%
%   Definition per MATLAB: the first and last R/2 fractions of the window
%   are cosine tapers; the middle is unity.
    arguments
        n
        r double = 0.5
    end
    w = ones(n, 1);
    if n > 1 && r > 0
        t = linspace(0, 1, n)';
        lo = t < r / 2;
        hi = t >= 1 - r / 2;
        w(lo) = 0.5 * (1 + cos(pi * (2 * t(lo) / r - 1)));
        w(hi) = 0.5 * (1 + cos(pi * (2 * t(hi) / r - 2 / r + 1)));
    end
end
