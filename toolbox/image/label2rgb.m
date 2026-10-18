function RGB = label2rgb(L, map)
%LABEL2RGB Convert a label matrix to a colour image.
%   RGB = LABEL2RGB(L) maps labels 1..max(L) to distinct colours generated
%   on the HSV wheel (MATLAB's default is the jet colormap; colours differ
%   but are likewise distinct).
%   RGB = LABEL2RGB(L,MAP) uses the N-by-3 colour map MAP.
    arguments
        L
        map = []
    end
    sz = size(L);
    nmax = round(max(max(L)));
    if nmax < 1
        nmax = 1;
    end
    if isempty(map)
        map = zeros(nmax, 3);
        for k = 1:nmax
            h = mod((k - 1) * 0.6180339887, 1);
            map(k, :) = hsv2rgb([h, 0.65, 0.9]);
        end
    end
    nm = size(map, 1);
    RGB = zeros(sz(1), sz(2), 3);
    for i = 1:sz(1)
        for j = 1:sz(2)
            k = round(L(i, j));
            if k < 1
                k = 1;
            end
            if k > nm
                k = nm;
            end
            c = map(k, :);
            RGB(i, j, 1) = c(1);
            RGB(i, j, 2) = c(2);
            RGB(i, j, 3) = c(3);
        end
    end
end
