function [L, num] = bwlabel(BW, conn)
%BWLABEL Label connected components in a 2-D binary image.
%   [L,NUM] = BWLABEL(BW)   labels with 8-connectivity.
%   [L,NUM] = BWLABEL(BW,4) or (BW,8) selects the connectivity.
%   Labels are 1..NUM in raster order of first appearance.
    arguments
        BW
        conn double = 8
    end
    fg = BW ~= 0;
    sz = size(fg);
    m = sz(1);
    n = sz(2);
    L = zeros(m, n);
    next = 0;
    for i = 1:m
        for j = 1:n
            if fg(i, j)
                next = next + 1;
                L(i, j) = next;
            end
        end
    end
    changed = 1;
    while changed == 1
        changed = 0;
        for i = 1:m
            for j = 1:n
                if L(i, j) > 0
                    best = L(i, j);
                    if i > 1 && L(i - 1, j) > 0 && L(i - 1, j) < best
                        best = L(i - 1, j);
                    end
                    if i < m && L(i + 1, j) > 0 && L(i + 1, j) < best
                        best = L(i + 1, j);
                    end
                    if j > 1 && L(i, j - 1) > 0 && L(i, j - 1) < best
                        best = L(i, j - 1);
                    end
                    if j < n && L(i, j + 1) > 0 && L(i, j + 1) < best
                        best = L(i, j + 1);
                    end
                    if conn == 8
                        if i > 1 && j > 1 && L(i - 1, j - 1) > 0 && L(i - 1, j - 1) < best
                            best = L(i - 1, j - 1);
                        end
                        if i > 1 && j < n && L(i - 1, j + 1) > 0 && L(i - 1, j + 1) < best
                            best = L(i - 1, j + 1);
                        end
                        if i < m && j > 1 && L(i + 1, j - 1) > 0 && L(i + 1, j - 1) < best
                            best = L(i + 1, j - 1);
                        end
                        if i < m && j < n && L(i + 1, j + 1) > 0 && L(i + 1, j + 1) < best
                            best = L(i + 1, j + 1);
                        end
                    end
                    if best < L(i, j)
                        L(i, j) = best;
                        changed = 1;
                    end
                end
            end
        end
    end
    % compact labels to 1..num
    vals = zeros(next, 1);
    cnt = 0;
    for i = 1:m
        for j = 1:n
            v = L(i, j);
            if v > 0
                hit = 0;
                for k = 1:cnt
                    if vals(k) == v
                        hit = 1;
                    end
                end
                if hit == 0
                    cnt = cnt + 1;
                    vals(cnt) = v;
                end
            end
        end
    end
    vals = sort(vals(1:cnt));
    num = cnt;
    for i = 1:m
        for j = 1:n
            if L(i, j) > 0
                for k = 1:num
                    if vals(k) == L(i, j)
                        L(i, j) = k;
                    end
                end
            end
        end
    end
end
