function [p, F] = anova1(y, group)
%ANOVA1 One-way analysis of variance.
%   P = ANOVA1(Y,GROUP) tests the null hypothesis that the means of the
%   groups defined by GROUP are equal. P = ANOVA1(Y) treats each column of
%   the matrix Y as a group.
%   [P,F] = ANOVA1(...) also returns the F statistic (MATLAB additionally
%   returns the ANOVA table and a stats struct, unavailable in this kernel).
    arguments
        y
        group = []
    end
    if isempty(group)
        if size(y, 2) < 2
            error('anova1:group', 'provide a GROUP vector for vector data');
        end
        a = size(y, 2);
        n = size(y, 1);
        N = a * n;
        means = zeros(1, a);
        grand = 0;
        for j = 1:a
            s = 0;
            for i = 1:n
                s = s + y(i, j);
            end
            means(j) = s / n;
            grand = grand + s;
        end
        grand = grand / N;
        ssb = 0;
        ssw = 0;
        for j = 1:a
            ssb = ssb + n * (means(j) - grand) ^ 2;
            for i = 1:n
                ssw = ssw + (y(i, j) - means(j)) ^ 2;
            end
        end
    else
        y = y(:);
        group = group(:);
        N = numel(y);
        a = 0;
        for i = 1:N
            if group(i) > a
                a = group(i);
            end
        end
        counts = zeros(1, a);
        means = zeros(1, a);
        for j = 1:a
            s = 0;
            c = 0;
            for i = 1:N
                if group(i) == j
                    s = s + y(i);
                    c = c + 1;
                end
            end
            counts(j) = c;
            if c > 0
                means(j) = s / c;
            end
        end
        grand = 0;
        for i = 1:N
            grand = grand + y(i);
        end
        grand = grand / N;
        ssb = 0;
        ssw = 0;
        for j = 1:a
            if counts(j) > 0
                ssb = ssb + counts(j) * (means(j) - grand) ^ 2;
                for i = 1:N
                    if group(i) == j
                        ssw = ssw + (y(i) - means(j)) ^ 2;
                    end
                end
            end
        end
    end
    dfb = a - 1;
    dfw = N - a;
    if ssw <= 0
        F = Inf;
        p = 0;
    else
        F = (ssb / dfb) / (ssw / dfw);
        p = 1 - fcdf(F, dfb, dfw);
    end
end
