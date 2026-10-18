function [x, fval] = particleswarm(fun, nvars, lb, ub, options)
%PARTICLESWARM Particle swarm optimization for bounded minimization.
%   X = PARTICLESWARM(FUN,NVARS,LB,UB) minimizes FUN over NVARS variables.
%
%   Global-best PSO with inertia weight
%   (Kennedy & Eberhart, Proc. IEEE ICNN, 1995).
    arguments
        fun
        nvars
        lb = []
        ub = []
        options = []
    end
    lo = -10 * ones(nvars, 1);
    hi = 10 * ones(nvars, 1);
    if ~isempty(lb)
        if numel(lb) == 1, lo = lb * ones(nvars, 1); else, lo = lb(:); end
    end
    if ~isempty(ub)
        if numel(ub) == 1, hi = ub * ones(nvars, 1); else, hi = ub(:); end
    end

    np = 12;
    maxit = 50;
    w = 0.72;
    c1 = 1.49;
    c2 = 1.49;
    pos = zeros(np, nvars);
    vel = zeros(np, nvars);
    for i = 1:np
        for j = 1:nvars
            pos(i, j) = lo(j) + (hi(j) - lo(j)) * rand;
            vel(i, j) = 0.1 * (hi(j) - lo(j)) * randn;
        end
    end
    fit = zeros(np, 1);
    for i = 1:np
        fit(i) = fun(pos(i, :)');
    end
    pbest = pos;
    pbestf = fit;
    [gbestf, gi] = min(fit);
    gbest = pos(gi, :);

    for it = 1:maxit
        for i = 1:np
            for j = 1:nvars
                r1 = rand;
                r2 = rand;
                vel(i, j) = w * vel(i, j) + c1 * r1 * (pbest(i, j) - pos(i, j)) ...
                    + c2 * r2 * (gbest(j) - pos(i, j));
                v = pos(i, j) + vel(i, j);
                if v < lo(j), v = lo(j); end
                if v > hi(j), v = hi(j); end
                pos(i, j) = v;
            end
            f = fun(pos(i, :)');
            fit(i) = f;
            if f < pbestf(i)
                pbestf(i) = f;
                pbest(i, :) = pos(i, :);
                if f < gbestf
                    gbestf = f;
                    gbest = pos(i, :);
                end
            end
        end
    end
    x = gbest';
    fval = gbestf;
end
