function [x, fval, exitflag] = fmincon(fun, x0, A, b, Aeq, beq, lb, ub, nonlcon, options)
%FMINCON Constrained nonlinear minimization (exterior penalty + pattern search).
%   X = FMINCON(FUN,X0,A,B) minimizes FUN subject to A*X <= B.
%   X = FMINCON(FUN,X0,A,B,AEQ,BEQ,LB,UB) also honors equalities and bounds.
%
%   Implementation: sequential exterior penalty with a Hooke-Jeeves inner
%   pattern search (Nocedal & Wright, Numerical Optimization, 2nd ed., ch.17;
%   Hooke & Jeeves, J. ACM 8(2), 1961).
%   Non-linear constraints (NONLCON) are not supported by this kernel build.
    arguments
        fun
        x0
        A = []
        b = []
        Aeq = []
        beq = []
        lb = []
        ub = []
        nonlcon = []
        options = []
    end
    n = numel(x0);
    x = x0(:);
    lo = -1e6 * ones(n, 1);
    hi = 1e6 * ones(n, 1);
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

    mu = 10;
    for stage = 1:4
        step = 1;
        fbest = 1e300;
        for it = 1:150
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
                        if p < fbest
                            fbest = p;
                            x = cand;
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
        mu = mu * 10;
    end

    % Exact feasibility projection for linear equalities.
    if ~isempty(Aeq)
        r = Aeq * x - beq;
        try
            corr = Aeq' * ((Aeq * Aeq') \ r);
        catch
            corr = Aeq' * pinv(Aeq * Aeq') * r;
        end
        x = x - corr;
    end

    x = x(:);
    fval = fun(x);
    viol = 0;
    if ~isempty(A)
        v = A * x - b;
        for k = 1:numel(v)
            if v(k) > viol
                viol = v(k);
            end
        end
    end
    if ~isempty(Aeq)
        v = abs(Aeq * x - beq);
        for k = 1:numel(v)
            if v(k) > viol
                viol = v(k);
            end
        end
    end
    for k = 1:n
        if lo(k) > -1e5 && lo(k) - x(k) > viol
            viol = lo(k) - x(k);
        end
        if hi(k) < 1e5 && x(k) - hi(k) > viol
            viol = x(k) - hi(k);
        end
    end
    if viol <= 1e-6
        exitflag = 1;
    else
        exitflag = -2;
    end
end
