function beta = nlinfit(X, y, modelfun, beta0)
%NLINFIT Nonlinear least-squares regression (Levenberg-Marquardt).
%   BETA = NLINFIT(X,Y,MODELFUN,BETA0) fits Y = MODELFUN(BETA,X) with a
%   numerical-Jacobian Levenberg-Marquardt iteration (Nocedal & Wright,
%   Numerical Optimization, 2nd ed., ch. 10).
%
%   The damping term keeps the iteration finite; any non-finite model value
%   is reported as an error instead of being passed to the linear solver.
    arguments
        X
        y
        modelfun
        beta0
    end
    y = reshape(y, numel(y), 1);
    m = numel(y);
    beta = reshape(beta0, numel(beta0), 1);
    nb = numel(beta);
    if m == 0 || nb == 0
        error('nlinfit:emptyInput', 'X, y and beta0 must be non-empty');
    end
    f = modelfun(beta, X);
    f = reshape(f, numel(f), 1);
    if numel(f) ~= m
        error('nlinfit:WrongSizeFunOutput', 'modelfun must return a vector the same size as y');
    end
    r = y - f;
    cost = sum(r .* r);
    if ~isfinite(cost)
        error('nlinfit:nonfinite', 'model function returned non-finite values');
    end
    lambda = 1e-3;
    for iter = 1:200
        J = zeros(m, nb);
        for j = 1:nb
            h = 1e-6 * (abs(beta(j)) + 1);
            bp = beta;
            bp(j) = bp(j) + h;
            fp = modelfun(bp, X);
            fp = reshape(fp, numel(fp), 1);
            if numel(fp) ~= m
                error('nlinfit:WrongSizeFunOutput', 'modelfun must return a vector the same size as y');
            end
            for i = 1:m
                J(i, j) = (fp(i) - f(i)) / h;
            end
        end
        if ~all(isfinite(J(:))) || ~all(isfinite(r))
            error('nlinfit:nonfinite', 'model function returned non-finite values');
        end
        JT = J';
        A = JT * J;
        g = JT * r;
        accepted = 0;
        step = zeros(nb, 1);
        for tries = 1:20
            M = A;
            for j = 1:nb
                M(j, j) = M(j, j) + lambda * (A(j, j) + 1e-12);
            end
            step = M \ g;
            if ~all(isfinite(step))
                lambda = lambda * 10;
            else
                bt = beta + step;
                ft = modelfun(bt, X);
                ft = reshape(ft, numel(ft), 1);
                if numel(ft) ~= m
                    error('nlinfit:WrongSizeFunOutput', 'modelfun must return a vector the same size as y');
                end
                rt = y - ft;
                ct = sum(rt .* rt);
                if isfinite(ct) && ct < cost
                    beta = bt;
                    f = ft;
                    r = rt;
                    cost = ct;
                    lambda = max(lambda / 10, 1e-12);
                    accepted = 1;
                    break
                else
                    lambda = lambda * 10;
                end
            end
        end
        if accepted == 0
            break
        end
        if max(abs(step)) < 1e-12 * (max(abs(beta)) + 1)
            break
        end
    end
end
