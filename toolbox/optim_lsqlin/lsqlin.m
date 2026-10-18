function [x, resnorm, residual, exitflag] = lsqlin(C, d, A, b, Aeq, beq, lb, ub, x0, options)
%LSQLIN Constrained linear least squares.
%   X = LSQLIN(C,D,A,B) minimizes NORM(C*X-D) subject to A*X <= B.
%   X = LSQLIN(C,D,A,B,AEQ,BEQ,LB,UB) also honors equalities and bounds.
%
%   Algorithm: primal active-set method for the convex QP
%   min 0.5*||C*x-d||^2 (Goldfarb & Idnani, Math. Prog. 27, 1983).
    arguments
        C
        d
        A = []
        b = []
        Aeq = []
        beq = []
        lb = []
        ub = []
        x0 = []
        options = []
    end
    n = size(C, 2);
    d = d(:);

    Aall = [];
    ball = [];
    me = 0;
    if ~isempty(Aeq)
        Aall = Aeq;
        if numel(beq) == 1
            ball = zeros(size(Aeq, 1), 1);
            ball(1) = beq;
        else
            ball = beq(:);
        end
        me = size(Aeq, 1);
    end
    if ~isempty(A)
        Aall = [Aall; A];
        if numel(b) == 1
            bt = zeros(size(A, 1), 1);
            bt(1) = b;
            ball = [ball; bt];
        else
            ball = [ball; b(:)];
        end
    end
    if ~isempty(lb)
        Aall = [Aall; -eye(n)];
        if numel(lb) == 1
            ball = [ball; -lb * ones(n, 1)];
        else
            ball = [ball; -lb(:)];
        end
    end
    if ~isempty(ub)
        Aall = [Aall; eye(n)];
        if numel(ub) == 1
            ball = [ball; ub * ones(n, 1)];
        else
            ball = [ball; ub(:)];
        end
    end

    H = C' * C;
    g0 = C' * d;

    if isempty(Aall)
        xr = C \ d;
        x = zeros(n, 1);
        for i = 1:n
            x(i) = xr(i);
        end
        residual = C * x - d;
        resnorm = sum(residual .* residual);
        exitflag = 1;
        return
    end

    W = [];
    for k = 1:me
        W = [W; k];
    end

    x = C \ d;
    if ~isempty(x0)
        x = x0(:);
    end

    tol = 1e-9;
    exitflag = 0;
    for iter = 1:100
        mW = numel(W);
        if mW == 0
            xr = C \ d;
            x = zeros(n, 1);
            for i = 1:n
                x(i) = xr(i);
            end
            lam = [];
        else
            AW = Aall(W, :);
            bW = ball(W);
            m = n + mW;
            K = zeros(m, m);
            for i = 1:n
                for j = 1:n
                    K(i, j) = H(i, j);
                end
            end
            rhs = zeros(m, 1);
            for i = 1:n
                rhs(i) = g0(i);
            end
            for i = 1:mW
                for j = 1:n
                    K(n + i, j) = AW(i, j);
                    K(j, n + i) = AW(i, j);
                end
                rhs(n + i) = bW(i);
            end
            try
                sol = K \ rhs;
            catch
                sol = pinv(K) * rhs;
            end
            x = zeros(n, 1);
            for i = 1:n
                x(i) = sol(i);
            end
            lam = zeros(mW, 1);
            for i = 1:mW
                lam(i) = sol(n + i);
            end
        end

        % Drop the most negative inequality multiplier.
        drop = 0;
        worst = -tol;
        for k = 1:numel(W)
            if W(k) > me
                if lam(k) < worst
                    worst = lam(k);
                    drop = k;
                end
            end
        end
        if drop > 0
            W2 = [];
            for k = 1:numel(W)
                if k ~= drop
                    W2 = [W2; W(k)];
                end
            end
            W = W2;
            exitflag = 0;
            continue
        end

        % Add the most violated constraint.
        r = Aall * x - ball;
        worstv = tol;
        add = 0;
        for k = 1:numel(r)
            inW = 0;
            for j = 1:numel(W)
                if W(j) == k
                    inW = 1;
                end
            end
            if inW == 0
                if r(k) > worstv
                    worstv = r(k);
                    add = k;
                end
            end
        end
        if add == 0
            exitflag = 1;
            break
        end
        W = [W; add];
    end

    residual = C * x - d;
    resnorm = sum(residual .* residual);
end
