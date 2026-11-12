function B = medfilt2(A, m, n)
%MEDFILT2 2-D median filtering.
%   B = MEDFILT2(A)      filters A with a 3-by-3 median window.
%   B = MEDFILT2(A,[M N]) or MEDFILT2(A,M,N) uses an M-by-N window.
%   Borders are zero padded (MATLAB's default).  For an even window the
%   neighbourhood spans rows i..i+M-1 (no half-pixel centring), matching
%   MATLAB's ORDFILT2-based implementation.
    arguments
        A
        m = 3
        n = 3
    end
    if numel(m) == 2
        n = m(2);
        m = m(1);
    end
    if m < 1 || n < 1
        error('medfilt2:expectedPositive', 'the neighbourhood size must be positive');
    end
    sz = size(A);
    rows = sz(1);
    cols = sz(2);
    B = zeros(rows, cols);
    om = floor((m - 1) / 2);
    on = floor((n - 1) / 2);
    for i = 1:rows
        for j = 1:cols
            vals = zeros(m * n, 1);
            k = 0;
            for a = 0:(m - 1)
                for b = 0:(n - 1)
                    k = k + 1;
                    ii = i - om + a;
                    jj = j - on + b;
                    if ii >= 1 && ii <= rows && jj >= 1 && jj <= cols
                        vals(k) = A(ii, jj);
                    end
                end
            end
            B(i, j) = median(vals);
        end
    end
end
