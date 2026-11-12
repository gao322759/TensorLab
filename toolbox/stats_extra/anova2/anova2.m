function p = anova2(y, reps)
%ANOVA2 Two-way analysis of variance with replication.
%   P = ANOVA2(Y,REPS) returns [P_COLUMNS P_ROWS] when REPS = 1 and
%   [P_COLUMNS P_ROWS P_INTERACTION] otherwise, matching MATLAB's order.
    arguments
        y
        reps double = 1
    end
    if reps < 1 || reps ~= round(reps)
        error('anova2:invalidReps', 'REPS must be a positive integer');
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
    % MATLAB 的输出顺序为 [p_columns p_rows (p_interaction)]
    if reps == 1
        % 无重复的两因素方差分析：交互项充当误差项
        p = zeros(1, 2);
        if dfab > 0 && ssab > 0
            mse = ssab / dfab;
            p(1) = 1 - fcdf((ssb / dfb) / mse, dfb, dfab);
            p(2) = 1 - fcdf((ssa / dfa) / mse, dfa, dfab);
        end
    else
        p = zeros(1, 3);
        if dfe > 0 && sse > 0
            mse = sse / dfe;
            p(1) = 1 - fcdf((ssb / dfb) / mse, dfb, dfe);
            p(2) = 1 - fcdf((ssa / dfa) / mse, dfa, dfe);
            p(3) = 1 - fcdf((ssab / dfab) / mse, dfab, dfe);
        end
    end
end
