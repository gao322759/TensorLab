function [x, fval] = simulannealbnd(fun, x0, lb, ub, options)
%SIMULANNEALBND Simulated annealing for bounded minimization.
%   X = SIMULANNEALBND(FUN,X0,LB,UB) minimizes FUN starting at X0.
%
%   Geometric cooling schedule RAND-based Metropolis acceptance
%   (Kirkpatrick, Gelatt & Vecchi, Science 220, 1983).
    arguments
        fun
        x0
        lb = []
        ub = []
        options = []
    end
    n = numel(x0);
    x = x0(:);
    lo = -10 * ones(n, 1);
    hi = 10 * ones(n, 1);
    if ~isempty(lb)
        if numel(lb) == 1, lo = lb * ones(n, 1); else, lo = lb(:); end
    end
    if ~isempty(ub)
        if numel(ub) == 1, hi = ub * ones(n, 1); else, hi = ub(:); end
    end
    for k = 1:n
        if x(k) < lo(k), x(k) = lo(k); end
        if x(k) > hi(k), x(k) = hi(k); end
    end

    f = fun(x);
    xb = x;
    fb = f;
    T = 1;
    maxit = 2000;
    for it = 1:maxit
        T = 1 * (0.999 ^ it);
        cand = zeros(n, 1);
        for k = 1:n
            step = 0.1 * (hi(k) - lo(k)) * sqrt(T) * randn;
            v = x(k) + step;
            if v < lo(k), v = lo(k); end
            if v > hi(k), v = hi(k); end
            cand(k) = v;
        end
        fc = fun(cand);
        if fc < f
            accept = 1;
        else
            if rand < exp(-(fc - f) / (T + 1e-12))
                accept = 1;
            else
                accept = 0;
            end
        end
        if accept == 1
            x = cand;
            f = fc;
            if fc < fb
                xb = cand;
                fb = fc;
            end
        end
    end
    x = xb;
    fval = fb;
end
