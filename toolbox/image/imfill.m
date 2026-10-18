function BW2 = imfill(BW, mode)
%IMFILL Fill holes in a binary image.
%   BW2 = IMFILL(BW,'holes') fills background regions not reachable from the
%   image border (4-connected flood fill).
    arguments
        BW
        mode = 'holes'
    end
    fg = BW ~= 0;
    sz = size(fg);
    m = sz(1);
    n = sz(2);
    P = zeros(m + 2, n + 2);
    P(2:m + 1, 2:n + 1) = fg;
    reach = zeros(m + 2, n + 2);
    cap = (m + 2) * (n + 2);
    si = zeros(cap, 1);
    sj = zeros(cap, 1);
    top = 1;
    si(1) = 1;
    sj(1) = 1;
    reach(1, 1) = 1;
    while top > 0
        i = si(top);
        j = sj(top);
        top = top - 1;
        if i > 1 && P(i - 1, j) == 0 && reach(i - 1, j) == 0
            top = top + 1; si(top) = i - 1; sj(top) = j; reach(i - 1, j) = 1;
        end
        if i < m + 2 && P(i + 1, j) == 0 && reach(i + 1, j) == 0
            top = top + 1; si(top) = i + 1; sj(top) = j; reach(i + 1, j) = 1;
        end
        if j > 1 && P(i, j - 1) == 0 && reach(i, j - 1) == 0
            top = top + 1; si(top) = i; sj(top) = j - 1; reach(i, j - 1) = 1;
        end
        if j < n + 2 && P(i, j + 1) == 0 && reach(i, j + 1) == 0
            top = top + 1; si(top) = i; sj(top) = j + 1; reach(i, j + 1) = 1;
        end
    end
    BW2 = zeros(m, n);
    for i = 1:m
        for j = 1:n
            if fg(i, j)
                BW2(i, j) = 1;
            elseif reach(i + 1, j + 1) == 0
                BW2(i, j) = 1;
            end
        end
    end
end
