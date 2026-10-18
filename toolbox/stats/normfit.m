function [muhat, sigmahat, muci, sigmaci] = normfit(x, alpha)
%NORMFIT Normal parameter estimates with confidence intervals.
%   [MUHAT,SIGMAHAT] = NORMFIT(X) returns the MLE-style estimates (mean and
%   sample standard deviation, N-1 normalization, as MATLAB).
%   [MUHAT,SIGMAHAT,MUCI,SIGMACI] = NORMFIT(X,ALPHA) also returns
%   100*(1-ALPHA)% confidence intervals for mu and sigma.
    arguments
        x
        alpha double = 0.05
    end
    x = reshape(x, numel(x), 1);
    n = numel(x);
    muhat = mean(x);
    sigmahat = std(x);
    tc = tinv(1 - alpha / 2, n - 1);
    se = sigmahat / sqrt(n);
    muci = [muhat - tc * se, muhat + tc * se];
    lo = chi2inv(alpha / 2, n - 1);
    hi = chi2inv(1 - alpha / 2, n - 1);
    sigmaci = [sqrt((n - 1) * sigmahat^2 / hi), sqrt((n - 1) * sigmahat^2 / lo)];
end
