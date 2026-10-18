function CC = bwconncomp(BW, conn)
%BWCONNCOMP Connected components of a binary image.
%   CC = BWCONNCOMP(BW) returns a struct with fields:
%     Connectivity  (4 or 8, default 8)
%     ImageSize     ([rows cols])
%     NumObjects    (number of components)
%     PixelIdxList  (1-by-NumObjects cell of column-major linear indices)
    arguments
        BW
        conn double = 8
    end
    [L, num] = bwlabel(BW, conn);
    sz = size(BW);
    m = sz(1);
    n = sz(2);
    pix = cell(1, num);
    for k = 1:num
        idxs = [];
        for j = 1:n
            for i = 1:m
                if L(i, j) == k
                    idxs = [idxs; (j - 1) * m + i];
                end
            end
        end
        pix{k} = idxs;
    end
    CC = struct('Connectivity', conn, 'ImageSize', {sz}, ...
        'NumObjects', num, 'PixelIdxList', {pix});
end
