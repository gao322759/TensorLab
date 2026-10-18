function B = imtranslate(A, t)
%IMTRANSLATE Translate an image by an integer offset.
%   B = IMTRANSLATE(A,[DX DY]) shifts A by DX columns and DY rows
%   (positive = right/down); vacated pixels are zero.
    arguments
        A
        t
    end
    sz = size(A);
    m = sz(1);
    n = sz(2);
    p = size(A, 3);
    dx = round(t(1));
    dy = round(t(2));
    B = zeros(m, n, p);
    for c = 1:p
        for i = 1:m
            ii = i - dy;
            if ii >= 1 && ii <= m
                for j = 1:n
                    jj = j - dx;
                    if jj >= 1 && jj <= n
                        B(i, j, c) = A(ii, jj, c);
                    end
                end
            end
        end
    end
end
