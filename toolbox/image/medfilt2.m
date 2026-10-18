function B = medfilt2(A, m, n)
%MEDFILT2 2-D median filtering.
%   B = MEDFILT2(A)      filters A with a 3-by-3 median window.
%   B = MEDFILT2(A,[M N]) or MEDFILT2(A,M,N) uses an M-by-N window.
%   Borders are zero padded (MATLAB's default).
    arguments
        A
        m = 3
        n = 3
    end
    if numel(m) == 2
        n = m(2);
        m = m(1);
    end
    sz = size(A);
    rows = sz(1);
    cols = sz(2);
    B = zeros(rows, cols);
    hm = floor(m / 2);
    hn = floor(n / 2);
    for i = 1:rows
        for j = 1:cols
            vals = zeros(m * n, 1);
            k = 0;
            for a = -hm:hm
                for b = -hn:hn
                    k = k + 1;
                    ii = i + a;
                    jj = j + b;
                    if ii >= 1 && ii <= rows && jj >= 1 && jj <= cols
                        vals(k) = A(ii, jj);
                    end
                end
            end
            B(i, j) = median(vals);
        end
    end
end
