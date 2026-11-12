function [p, H] = kruskalwallis(x, group)
%KRUSKALWALLIS Kruskal-Wallis nonparametric one-way ANOVA.
%   P = KRUSKALWALLIS(X,GROUP) tests whether the group medians differ.
%   P = KRUSKALWALLIS(X) with a matrix X treats each column as a group.
%   [P,H] = KRUSKALWALLIS(...) also returns the H statistic.
    arguments
        x
        group = []
    end
    if isempty(group)
        nc = size(x, 2);
        if nc < 2
            error('kruskalwallis:group', 'provide a GROUP vector');
        end
        nr = size(x, 1);
        xs = zeros(nr * nc, 1);
        gs = zeros(nr * nc, 1);
        for j = 1:nc
            for i = 1:nr
                idx = (j - 1) * nr + i;
                xs(idx) = x(i, j);
                gs(idx) = j;
            end
        end
        x = xs;
        group = gs;
    end
    x = x(:);
    group = group(:);
    if numel(x) ~= numel(group)
        error('kruskalwallis:size', 'X and GROUP must have the same length');
    end
    n = numel(x);

    % Ranks with tie correction.
    [xs, idx] = sort(x);
    ranks = zeros(n, 1);
    i = 1;
    while i <= n
        j = i;
        while j < n && xs(j + 1) == xs(i)
            j = j + 1;
        end
        avg = (i + j) / 2;
        for k = i:j
            ranks(idx(k)) = avg;
        end
        i = j + 1;
    end

    k = 0;
    for i = 1:n
        if group(i) > k
            k = group(i);
        end
    end
    nj = zeros(1, k);
    rj = zeros(1, k);
    for j = 1:k
        for i = 1:n
            if group(i) == j
                nj(j) = nj(j) + 1;
                rj(j) = rj(j) + ranks(i);
            end
        end
    end
    s = 0;
    for j = 1:k
        if nj(j) > 0
            s = s + rj(j) ^ 2 / nj(j);
        end
    end
    H = 12 / (n * (n + 1)) * s - 3 * (n + 1);
    p = 1 - chi2cdf(H, k - 1);
end
