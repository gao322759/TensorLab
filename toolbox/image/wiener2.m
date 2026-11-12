function B = wiener2(A, m, n)
%WIENER2 2-D adaptive Wiener (local mean/variance) denoising.
%   B = WIENER2(A) filters A with a 3-by-3 neighbourhood.
%   B = WIENER2(A,[M N]) or WIENER2(A,M,N) uses an M-by-N window.
%
%   Matching MATLAB, the local statistics use zero padding, the noise power
%   is the mean of all local variances, and B = MU + max(0,VAR-NOISE)/VAR *
%   (A-MU).  Reference: Lim, Two-Dimensional Signal and Image Processing,
%   1990, ch. 9.
    arguments
        A
        m double = 3
        n double = 3
    end
    if numel(m) == 2
        n = m(2);
        m = m(1);
    end
    if m < 1 || n < 1 || m ~= 2 * floor(m / 2) + 1 || n ~= 2 * floor(n / 2) + 1
        error('wiener2:expectedOddPositive', 'the neighbourhood size must be odd and positive');
    end
    sz = size(A);
    rows = sz(1);
    cols = sz(2);
    hm = floor(m / 2);
    hn = floor(n / 2);
    P = zeros(rows + 2 * hm, cols + 2 * hn);
    for i = 1:rows
        for j = 1:cols
            P(i + hm, j + hn) = A(i, j);
        end
    end
    mu = zeros(rows, cols);
    va = zeros(rows, cols);
    for i = 1:rows
        for j = 1:cols
            s = 0;
            s2 = 0;
            for a = 1:m
                for b = 1:n
                    v = P(i + a - 1, j + b - 1);
                    s = s + v;
                    s2 = s2 + v * v;
                end
            end
            mu(i, j) = s / (m * n);
            va(i, j) = s2 / (m * n) - mu(i, j) * mu(i, j);
        end
    end
    noise = sum(sum(va)) / (rows * cols);
    B = zeros(rows, cols);
    for i = 1:rows
        for j = 1:cols
            if va(i, j) > 1e-12
                f = (va(i, j) - noise) / va(i, j);
                if f < 0
                    f = 0;
                end
                B(i, j) = mu(i, j) + f * (A(i, j) - mu(i, j));
            else
                B(i, j) = mu(i, j);
            end
        end
    end
end
