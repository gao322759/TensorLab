function [x, fval, exitflag, output] = intlinprog(f, intcon, A, b, Aeq, beq, lb, ub, options)
%INTLINPROG Mixed-integer linear programming (branch and bound).
%   X = INTLINPROG(F,INTCON,A,B) minimizes F'*X subject to A*X <= B with
%   the variables listed in INTCON restricted to integers.
%   X = INTLINPROG(F,INTCON,A,B,AEQ,BEQ,LB,UB) also honors equalities/bounds.
%
%   Algorithm: depth-first branch and bound; LP relaxations solved with the
%   kernel LINPROG (Land & Doig, Econometrica 28, 1960).
    arguments
        f
        intcon
        A = []
        b = []
        Aeq = []
        beq = []
        lb = []
        ub = []
        options = []
    end
    f = f(:);
    intcon = intcon(:);
    n = numel(f);
    if isempty(lb)
        lb = zeros(n, 1);
    elseif numel(lb) == 1
        lb = lb * ones(n, 1);
    else
        lb = lb(:);
    end
    if isempty(ub)
        ub = 1e12 * ones(n, 1);
    elseif numel(ub) == 1
        ub = ub * ones(n, 1);
    else
        ub = ub(:);
    end

    maxNodes = 500;
    Ls = zeros(n, maxNodes);
    Us = zeros(n, maxNodes);
    Ls(:, 1) = lb;
    Us(:, 1) = ub;
    top = 1;

    xbest = [];
    fbest = inf;
    nodes = 0;
    exitflag = 0;

    while top > 0
        lo = Ls(:, top);
        hi = Us(:, top);
        top = top - 1;
        nodes = nodes + 1;
        if nodes > maxNodes
            exitflag = 0;
            break
        end

        [xr, fr, flag] = linprog(f, A, b, Aeq, beq, lo, hi);
        if flag ~= 1
            continue
        end
        if fr >= fbest - 1e-9
            continue
        end

        % Most fractional integer variable.
        jf = 0;
        frac = 0;
        for k = 1:numel(intcon)
            v = xr(intcon(k));
            d = abs(v - round(v));
            if d > frac
                frac = d;
                jf = intcon(k);
            end
        end

        if frac <= 1e-6
            xbest = xr;
            fbest = fr;
            exitflag = 1;
        else
            vj = xr(jf);
            down = floor(vj);
            up = ceil(vj);

            % Right branch: x(jf) >= up
            if up <= hi(jf) && top < maxNodes
                top = top + 1;
                Ls(:, top) = lo;
                Us(:, top) = hi;
                Ls(jf, top) = up;
            end
            % Left branch: x(jf) <= down
            if down >= lo(jf) && top < maxNodes
                top = top + 1;
                Ls(:, top) = lo;
                Us(:, top) = hi;
                Us(jf, top) = down;
            end
        end
    end

    if isempty(xbest)
        x = [];
        fval = inf;
        exitflag = -2;
    else
        x = xbest;
        fval = fbest;
    end
    output = nodes;
end
