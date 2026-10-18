function B = imdilate(A, SE)
%IMDILATE Morphological dilation.
%   B = IMDILATE(A,SE) dilates A with the structuring element SE (numeric
%   matrix; non-zero entries define the neighbourhood).  SE is centred using
%   floor(size/2) offsets, matching MATLAB.  Borders are treated as 0.
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
    B = zeros(sz(1), sz(2));
    for i = 1:sz(1)
        for j = 1:sz(2)
            v = 0;
            for a = 1:ss(1)
                for b = 1:ss(2)
                    if SE(a, b) ~= 0
                        ii = i + a - 1 - hm;
                        jj = j + b - 1 - hn;
                        if ii >= 1 && ii <= sz(1) && jj >= 1 && jj <= sz(2)
                            if A(ii, jj) > v
                                v = A(ii, jj);
                            end
                        end
                    end
                end
            end
            B(i, j) = v;
        end
    end
end
