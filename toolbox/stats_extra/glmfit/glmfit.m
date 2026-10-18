function [b, dev] = glmfit(X, y, distr)
%GLMFIT Generalized linear model regression.
%   B = GLMFIT(X,Y,DISTR) fits y = g^-1(X*B) with an intercept column.
%   DISTR is 'normal' (default), 'binomial' or 'poisson'.
%   [B,DEV] = GLMFIT(...) also returns the deviance.
%
%   Iteratively reweighted least squares (McCullagh & Nelder, Generalized
%   Linear Models, 2nd ed., 1989, ch. 2).
    arguments
        X
        y
        distr = 'normal'
    end
    n = numel(y);
    p = size(X, 2);
    if size(X, 1) ~= n
        error('glmfit:shape', 'x and y must have the same number of rows');
    end
    Xd = zeros(n, p + 1);
    for i = 1:n
        Xd(i, 1) = 1;
        for j = 1:p
            Xd(i, j + 1) = X(i, j);
        end
    end
    nb = p + 1;

    if strcmp(distr, 'normal')
        br = Xd \ y(:);
        b = zeros(nb, 1);
        for j = 1:nb
            b(j) = br(j);
        end
        rss = 0;
        for i = 1:n
            eta = 0;
            for j = 1:nb
                eta = eta + Xd(i, j) * b(j);
            end
            rss = rss + (y(i) - eta) ^ 2;
        end
        dev = rss;
        return
    end

    b = zeros(nb, 1);
    for iter = 1:100
        w = zeros(n, 1);
        z = zeros(n, 1);
        for i = 1:n
            eta = 0;
            for j = 1:nb
                eta = eta + Xd(i, j) * b(j);
            end
            if strcmp(distr, 'binomial')
                mu = 1 / (1 + exp(-eta));
                if mu < 1e-12
                    mu = 1e-12;
                end
                if mu > 1 - 1e-12
                    mu = 1 - 1e-12;
                end
                w(i) = mu * (1 - mu);
                z(i) = eta + (y(i) - mu) / w(i);
            elseif strcmp(distr, 'poisson')
                mu = exp(eta);
                if mu < 1e-12
                    mu = 1e-12;
                end
                w(i) = mu;
                z(i) = eta + (y(i) - mu) / mu;
            else
                error('glmfit:distr', 'unsupported distribution');
            end
        end
        XtWX = zeros(nb, nb);
        XtWz = zeros(nb, 1);
        for i = 1:n
            for j = 1:nb
                XtWz(j) = XtWz(j) + Xd(i, j) * w(i) * z(i);
                for k = 1:nb
                    XtWX(j, k) = XtWX(j, k) + Xd(i, j) * w(i) * Xd(i, k);
                end
            end
        end
        try
            br = XtWX \ XtWz;
        catch
            br = pinv(XtWX) * XtWz;
        end
        delta = 0;
        for j = 1:nb
            d = br(j) - b(j);
            b(j) = br(j);
            delta = delta + d * d;
        end
        if delta < 1e-10
            break
        end
    end

    dev = 0;
    for i = 1:n
        eta = 0;
        for j = 1:nb
            eta = eta + Xd(i, j) * b(j);
        end
        if strcmp(distr, 'binomial')
            mu = 1 / (1 + exp(-eta));
            if mu < 1e-12
                mu = 1e-12;
            end
            if mu > 1 - 1e-12
                mu = 1 - 1e-12;
            end
            if y(i) > 0
                dev = dev - 2 * y(i) * log(mu);
            end
            if y(i) < 1
                dev = dev - 2 * (1 - y(i)) * log(1 - mu);
            end
        else
            mu = exp(eta);
            if y(i) > 0
                dev = dev + 2 * (y(i) * log(y(i) / mu) - (y(i) - mu));
            else
                dev = dev + 2 * mu;
            end
        end
    end
end
