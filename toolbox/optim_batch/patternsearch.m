function [x, fval] = patternsearch(fun, x0, A, b, Aeq, beq, lb, ub, options)
%PATTERNSEARCH Pattern search (Hooke-Jeeves) for constrained minimization.
%   X = PATTERNSEARCH(FUN,X0) minimizes FUN starting at X0.
%   X = PATTERNSEARCH(FUN,X0,A,B,AEQ,BEQ,LB,UB) handles linear constraints
%   through an exterior penalty.
%
%   Exploratory moves along each coordinate with step halving
%   (Hooke & Jeeves, J. ACM 8(2), 1961).
    arguments
        fun
        x0
        A = []
        b = []
        Aeq = []
        beq = []
        lb = []
        ub = []
        options = []
    end
    n = numel(x0);
    x = x0(:);
    mu = 1e4;
    lo = -10 * ones(n, 1);
    hi = 10 * ones(n, 1);
    if ~isempty(lb)
        if numel(lb) == 1
            lo = lb * ones(n, 1);
        else
            lo = lb(:);
        end
    end
    if ~isempty(ub)
        if numel(ub) == 1
            hi = ub * ones(n, 1);
        else
            hi = ub(:);
        end
    end
    for k = 1:n
        if x(k) < lo(k)
            x(k) = lo(k);
        end
        if x(k) > hi(k)
            x(k) = hi(k);
        end
    end

    p = fun(x);
    if ~isempty(A)
        v = A * x - b;
        for j = 1:numel(v)
            if v(j) > 0
                p = p + mu * v(j) * v(j);
            end
        end
    end
    if ~isempty(Aeq)
        v = Aeq * x - beq;
        for j = 1:numel(v)
            p = p + mu * v(j) * v(j);
        end
    end
    f = p;

    step = 1;
    for it = 1:400
        improved = 0;
        for k = 1:n
            for s = 1:2
                cand = x;
                if s == 1
                    cand(k) = cand(k) + step;
                else
                    cand(k) = cand(k) - step;
                end
                if cand(k) >= lo(k) && cand(k) <= hi(k)
                    p = fun(cand);
                    if ~isempty(A)
                        v = A * cand - b;
                        for j = 1:numel(v)
                            if v(j) > 0
                                p = p + mu * v(j) * v(j);
                            end
                        end
                    end
                    if ~isempty(Aeq)
                        v = Aeq * cand - beq;
                        for j = 1:numel(v)
                            p = p + mu * v(j) * v(j);
                        end
                    end
                    if p < f
                        x = cand;
                        f = p;
                        improved = 1;
                    end
                end
            end
        end
        if improved == 0
            step = step * 0.5;
            if step < 1e-9
                break
            end
        end
    end
    fval = fun(x);
end
