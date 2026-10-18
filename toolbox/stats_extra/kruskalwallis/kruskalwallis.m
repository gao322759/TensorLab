function [p, H] = kruskalwallis(x, group)
%KRUSKALWALLIS Kruskal-Wallis nonparametric one-way ANOVA.
%   P = KRUSKALWALLIS(X,GROUP) tests whether the group medians differ.
%   [P,H] = KRUSKALWALLIS(...) also returns the H statistic.
    arguments
        x
        group = []
    end
    x = x(:);
    n = numel(x);
    if isempty(group)
        if size(x, 2) < 2
            error('kruskalwallis:group', 'provide a GROUP vector');
        end
        group = zeros(n, 1);
        for i = 1:n
            group(i) = 1;
        end
    end
    group = group(:);

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
