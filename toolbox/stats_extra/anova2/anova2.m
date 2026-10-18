function p = anova2(y, reps)
%ANOVA2 Two-way analysis of variance with replication.
%   P = ANOVA2(Y,REPS) returns [P_A P_B P_AB] for the factors A (rows),
%   B (columns, grouped in blocks of REPS columns) and their interaction.
%   Default REPS = 1 (no replication).
    arguments
        y
        reps double = 1
    end
    a = size(y, 1);
    nc = size(y, 2);
    b = nc / reps;
    if b ~= floor(b)
        error('anova2:shape', 'number of columns must be a multiple of REPS');
    end
    N = a * b * reps;
    grand = 0;
    for i = 1:a
        for j = 1:nc
            grand = grand + y(i, j);
        end
    end
    grand = grand / N;

    meanA = zeros(1, a);
    for i = 1:a
        s = 0;
        for j = 1:nc
            s = s + y(i, j);
        end
        meanA(i) = s / (b * reps);
    end
    meanB = zeros(1, b);
    for j = 1:b
        s = 0;
        for i = 1:a
            for k = 1:reps
                s = s + y(i, (j - 1) * reps + k);
            end
        end
        meanB(j) = s / (a * reps);
    end
    cell = zeros(a, b);
    for i = 1:a
        for j = 1:b
            s = 0;
            for k = 1:reps
                s = s + y(i, (j - 1) * reps + k);
            end
            cell(i, j) = s / reps;
        end
    end

    ssa = 0;
    for i = 1:a
        ssa = ssa + b * reps * (meanA(i) - grand) ^ 2;
    end
    ssb = 0;
    for j = 1:b
        ssb = ssb + a * reps * (meanB(j) - grand) ^ 2;
    end
    ssab = 0;
    sse = 0;
    for i = 1:a
        for j = 1:b
            ssab = ssab + reps * (cell(i, j) - meanA(i) - meanB(j) + grand) ^ 2;
            for k = 1:reps
                sse = sse + (y(i, (j - 1) * reps + k) - cell(i, j)) ^ 2;
            end
        end
    end

    dfa = a - 1;
    dfb = b - 1;
    dfab = (a - 1) * (b - 1);
    dfe = a * b * (reps - 1);
    p = zeros(1, 3);
    if dfe > 0 && sse > 0
        mse = sse / dfe;
        fa = (ssa / dfa) / mse;
        fb = (ssb / dfb) / mse;
        fab = (ssab / dfab) / mse;
        p(1) = 1 - fcdf(fa, dfa, dfe);
        p(2) = 1 - fcdf(fb, dfb, dfe);
        p(3) = 1 - fcdf(fab, dfab, dfe);
    end
end
