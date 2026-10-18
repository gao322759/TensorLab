function beta = nlinfit(X, y, modelfun, beta0)
%NLINFIT Nonlinear least-squares regression (Gauss-Newton).
%   BETA = NLINFIT(X,Y,MODELFUN,BETA0) fits Y = MODELFUN(BETA,X) by
%   numerical-Jacobian Gauss-Newton iterations (Nocedal & Wright, ch. 10).
    arguments
        X
        y
        modelfun
        beta0
    end
    y = y(:);
    m = numel(y);
    beta = beta0(:);
    nb = numel(beta);
    for iter = 1:100
        f = modelfun(beta, X);
        r = zeros(m, 1);
        for i = 1:m
            r(i) = y(i) - f(i);
        end
        J = zeros(m, nb);
        for j = 1:nb
            h = 1e-6 * (abs(beta(j)) + 1);
            bp = beta;
            bp(j) = bp(j) + h;
            fp = modelfun(bp, X);
            for i = 1:m
                J(i, j) = (fp(i) - f(i)) / h;
            end
        end
        try
            stepr = J \ r;
        catch
            stepr = pinv(J) * r;
        end
        delta = 0;
        for j = 1:nb
            d = stepr(j);
            beta(j) = beta(j) + d;
            delta = delta + d * d;
        end
        if delta < 1e-12
            break
        end
    end
end
