function [x, fval] = ga(fun, nvars, A, b, Aeq, beq, lb, ub, nonlcon, options)
%GA Genetic algorithm for bounded nonlinear minimization.
%   X = GA(FUN,NVARS) minimizes FUN over NVARS variables.
%   X = GA(FUN,NVARS,A,B,AEQ,BEQ,LB,UB) honors linear constraints via an
%   exterior penalty.
%
%   Real-coded GA: tournament selection, blend crossover, Gaussian mutation,
%   elitism (Holland, Adaptation in Natural and Artificial Systems, 1975).
    arguments
        fun
        nvars
        A = []
        b = []
        Aeq = []
        beq = []
        lb = []
        ub = []
        nonlcon = []
        options = []
    end
    popsize = 20;
    maxgen = 60;
    mu = 1e3;
    lo = -10 * ones(nvars, 1);
    hi = 10 * ones(nvars, 1);
    if ~isempty(lb)
        if numel(lb) == 1
            lo = lb * ones(nvars, 1);
        else
            lo = lb(:);
        end
    end
    if ~isempty(ub)
        if numel(ub) == 1
            hi = ub * ones(nvars, 1);
        else
            hi = ub(:);
        end
    end

    pop = zeros(popsize, nvars);
    for i = 1:popsize
        for j = 1:nvars
            pop(i, j) = lo(j) + (hi(j) - lo(j)) * rand;
        end
    end
    fit = zeros(popsize, 1);
    for i = 1:popsize
        z = pop(i, :)';
        p = fun(z);
        if ~isempty(A)
            v = A * z - b;
            for j = 1:numel(v)
                if v(j) > 0
                    p = p + mu * v(j) * v(j);
                end
            end
        end
        if ~isempty(Aeq)
            v = Aeq * z - beq;
            for j = 1:numel(v)
                p = p + mu * v(j) * v(j);
            end
        end
        fit(i) = p;
    end

    best = inf;
    stall = 0;
    for gen = 1:maxgen
        [fs, idx] = sort(fit);
        if fs(1) < best - 1e-8
            best = fs(1);
            stall = 0;
        else
            stall = stall + 1;
        end
        if stall >= 20
            break
        end

        newpop = zeros(popsize, nvars);
        newfit = zeros(popsize, 1);
        newpop(1, :) = pop(idx(1), :);
        newfit(1) = fit(idx(1));
        newpop(2, :) = pop(idx(2), :);
        newfit(2) = fit(idx(2));

        for i = 3:popsize
            p1 = 1 + floor(rand * popsize);
            p2 = 1 + floor(rand * popsize);
            parent1 = p1;
            if fit(p2) < fit(p1)
                parent1 = p2;
            end
            p3 = 1 + floor(rand * popsize);
            p4 = 1 + floor(rand * popsize);
            parent2 = p3;
            if fit(p4) < fit(p3)
                parent2 = p4;
            end

            child = zeros(1, nvars);
            for j = 1:nvars
                alpha = rand;
                v = alpha * pop(parent1, j) + (1 - alpha) * pop(parent2, j);
                if rand < 0.1
                    v = v + 0.1 * (hi(j) - lo(j)) * randn;
                end
                if v < lo(j)
                    v = lo(j);
                end
                if v > hi(j)
                    v = hi(j);
                end
                child(j) = v;
            end
            newpop(i, :) = child;

            z = child';
            p = fun(z);
            if ~isempty(A)
                v = A * z - b;
                for j = 1:numel(v)
                    if v(j) > 0
                        p = p + mu * v(j) * v(j);
                    end
                end
            end
            if ~isempty(Aeq)
                v = Aeq * z - beq;
                for j = 1:numel(v)
                    p = p + mu * v(j) * v(j);
                end
            end
            newfit(i) = p;
        end
        pop = newpop;
        fit = newfit;
    end

    [fval, idx] = min(fit);
    x = pop(idx, :)';
end
