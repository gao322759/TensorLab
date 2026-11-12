function B = imtranslate(A, t)
%IMTRANSLATE Translate an image by a sub-pixel offset.
%   B = IMTRANSLATE(A,[DX DY]) shifts A by DX columns and DY rows
%   (positive = right/down) using bilinear interpolation; vacated pixels
%   are zero, matching MATLAB's default.
    arguments
        A
        t
    end
    if numel(t) < 2
        error('imtranslate:invalidTranslation', 'T must have two elements, [DX DY]');
    end
    sz = size(A);
    m = sz(1);
    n = sz(2);
    p = size(A, 3);
    dx = t(1);
    dy = t(2);
    B = zeros(m, n, p);
    for c = 1:p
        P = A(:, :, c);
        for i = 1:m
            si = i - dy;
            i0 = floor(si);
            di = si - i0;
            for j = 1:n
                sj = j - dx;
                j0 = floor(sj);
                dj = sj - j0;
                v = 0;
                if i0 >= 1 && i0 <= m
                    if j0 >= 1 && j0 <= n
                        v = v + P(i0, j0) * (1 - di) * (1 - dj);
                    end
                    if j0 + 1 >= 1 && j0 + 1 <= n
                        v = v + P(i0, j0 + 1) * (1 - di) * dj;
                    end
                end
                if i0 + 1 >= 1 && i0 + 1 <= m
                    if j0 >= 1 && j0 <= n
                        v = v + P(i0 + 1, j0) * di * (1 - dj);
                    end
                    if j0 + 1 >= 1 && j0 + 1 <= n
                        v = v + P(i0 + 1, j0 + 1) * di * dj;
                    end
                end
                B(i, j, c) = v;
            end
        end
    end
end
