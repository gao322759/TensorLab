function J = histeq(I, n)
%HISTEQ Histogram equalization of an intensity image.
%   J = HISTEQ(I)    equalizes the intensities of I (assumed in [0,1]).
%   J = HISTEQ(I,N)  uses N discrete intensity levels (default 64).
    arguments
        I
        n double = 64
    end
    sz = size(I);
    Ic = I;
    for i = 1:sz(1)
        for j = 1:sz(2)
            v = Ic(i, j);
            if v < 0
                v = 0;
            end
            if v > 1
                v = 1;
            end
            Ic(i, j) = v;
        end
    end
    edges = linspace(0, 1, n + 1);
    counts = histcounts(Ic, edges);
    cdfv = cumsum(counts) / numel(Ic);
    J = zeros(sz(1), sz(2));
    for i = 1:sz(1)
        for j = 1:sz(2)
            b = ceil(Ic(i, j) * n);
            if b < 1
                b = 1;
            end
            if b > n
                b = n;
            end
            J(i, j) = cdfv(b);
        end
    end
end
