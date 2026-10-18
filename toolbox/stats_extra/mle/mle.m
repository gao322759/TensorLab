function phat = mle(x, distr)
%MLE Maximum likelihood parameter estimates.
%   PHAT = MLE(X) estimates the normal parameters [MU SIGMA].
%   PHAT = MLE(X,DISTR) supports 'normal', 'exponential' and 'poisson'.
%   Confidence intervals and censoring are not implemented.
    arguments
        x
        distr = 'normal'
    end
    x = x(:);
    n = numel(x);
    phat = 0;
    if strcmp(distr, 'normal')
        mu = 0;
        for i = 1:n
            mu = mu + x(i);
        end
        mu = mu / n;
        v = 0;
        for i = 1:n
            v = v + (x(i) - mu) ^ 2;
        end
        phat = zeros(1, 2);
        phat(1) = mu;
        phat(2) = sqrt(v / n);
    elseif strcmp(distr, 'exponential')
        mu = 0;
        for i = 1:n
            mu = mu + x(i);
        end
        phat = mu / n;
    elseif strcmp(distr, 'poisson')
        mu = 0;
        for i = 1:n
            mu = mu + x(i);
        end
        phat = mu / n;
    else
        error('mle:distr', 'unsupported distribution');
    end
end
