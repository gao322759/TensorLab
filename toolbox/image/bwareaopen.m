function BW2 = bwareaopen(BW, P, conn)
%BWARE AOPEN Remove small connected components from a binary image.
%   BW2 = BWAREAOPEN(BW,P) removes components with fewer than P pixels.
%   BW2 = BWAREAOPEN(BW,P,CONN) with connectivity 4 or 8 (default 8).
    arguments
        BW
        P double
        conn double = 8
    end
    sz = size(BW);
    [L, num] = bwlabel(BW, conn);
    BW2 = zeros(sz(1), sz(2));
    if num == 0
        return
    end
    areas = zeros(num, 1);
    for i = 1:sz(1)
        for j = 1:sz(2)
            k = L(i, j);
            if k > 0
                areas(k) = areas(k) + 1;
            end
        end
    end
    for i = 1:sz(1)
        for j = 1:sz(2)
            k = L(i, j);
            if k > 0 && areas(k) >= P
                BW2(i, j) = 1;
            end
        end
    end
end
