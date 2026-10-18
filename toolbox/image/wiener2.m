function B = wiener2(A, m, n)
%WIENER2 2-D adaptive Wiener (local mean/variance) denoising.
%   B = WIENER2(A) filters A with a 3-by-3 neighbourhood.
%   B = WIENER2(A,[M N]) or WIENER2(A,M,N) uses an M-by-N window.
%
%   The noise power is estimated as the mean of the local variances
%   (MATLAB's documented behaviour).  Reference: Lim, Two-Dimensional
%   Signal and Image Processing, 1990, ch. 9.
    arguments
        A
        m double = 3
        n double = 3
    end
    if numel(m) == 2
        n = m(2);
        m = m(1);
    end
    sz = size(A);
    hm = floor(m / 2);
    hn = floor(n / 2);
    mu = zeros(sz(1), sz(2));
    va = zeros(sz(1), sz(2));
    for i = 1:sz(1)
        for j = 1:sz(2)
            cnt = 0;
            s = 0;
            s2 = 0;
            for a = -hm:hm
                for b = -hn:hn
                    ii = i + a;
                    jj = j + b;
                    if ii >= 1 && ii <= sz(1) && jj >= 1 && jj <= sz(2)
                        v = A(ii, jj);
                        cnt = cnt + 1;
                        s = s + v;
                        s2 = s2 + v * v;
                    end
                end
            end
            mu(i, j) = s / cnt;
            va(i, j) = s2 / cnt - mu(i, j) * mu(i, j);
        end
    end
    noise = sum(sum(va)) / numel(va);
    B = zeros(sz(1), sz(2));
    for i = 1:sz(1)
        for j = 1:sz(2)
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
