function B = imerode(A, SE)
%IMERODE Morphological erosion.
%   B = IMERODE(A,SE) erodes A with the structuring element SE (numeric
%   matrix; non-zero entries define the neighbourhood).  Out-of-range
%   neighbours are ignored (MATLAB's border convention).
    arguments
        A
        SE
    end
    sz = size(A);
    if strcmp(class(SE), 'struct')
        SE = SE.Neighborhood;
    end
    ss = size(SE);
    hm = floor(ss(1) / 2);
    hn = floor(ss(2) / 2);
    big = max(max(A)) + 1;
    B = zeros(sz(1), sz(2));
    for i = 1:sz(1)
        for j = 1:sz(2)
            v = big;
            for a = 1:ss(1)
                for b = 1:ss(2)
                    if SE(a, b) ~= 0
                        ii = i + a - 1 - hm;
                        jj = j + b - 1 - hn;
                        if ii >= 1 && ii <= sz(1) && jj >= 1 && jj <= sz(2)
                            if A(ii, jj) < v
                                v = A(ii, jj);
                            end
                        end
                    end
                end
            end
            if v == big
                v = max(max(A));
            end
            B(i, j) = v;
        end
    end
end
