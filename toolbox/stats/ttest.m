function [h, p, ci] = ttest(x, m, alpha)
%TTEST One-sample t-test (subset of MATLAB's ttest; no stats struct).
%   H = TTEST(X)            tests the null hypothesis that X has mean 0.
%   H = TTEST(X,M)          tests the null hypothesis that X has mean M.
%   [H,P,CI] = TTEST(X,M,ALPHA) also returns the p-value and the
%   100*(1-ALPHA)% confidence interval for the mean.
%
%   Paired-sample tests are performed by passing the difference vector.
    arguments
        x
        m double = 0
        alpha double = 0.05
    end
    x = reshape(x, numel(x), 1);
    n = numel(x);
    xbar = mean(x);
    se = std(x) / sqrt(n);
    tval = (xbar - m) / se;
    df = n - 1;
    p = 2 * (1 - tcdf(abs(tval), df));
    h = p < alpha;
    tc = tinv(1 - alpha / 2, df);
    ci = [xbar - tc * se, xbar + tc * se];
end
