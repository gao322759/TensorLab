function [h, p, ci] = vartest(x, v, alpha)
%VARTEST One-sample chi-square test for a variance (subset).
%   H = VARTEST(X,V) tests the null hypothesis that X comes from a normal
%   distribution with variance V, against the two-sided alternative.
%   [H,P,CI] = VARTEST(X,V,ALPHA) also returns the p-value and the
%   100*(1-ALPHA)% confidence interval for the variance.
    arguments
        x
        v double
        alpha double = 0.05
    end
    x = reshape(x, numel(x), 1);
    n = numel(x);
    df = n - 1;
    s2 = var(x);
    chi2 = df * s2 / v;
    lo = chi2cdf(chi2, df);
    p = 2 * min(lo, 1 - lo);
    p = min(p, 1);
    h = p < alpha;
    cil = df * s2 / chi2inv(1 - alpha / 2, df);
    cih = df * s2 / chi2inv(alpha / 2, df);
    ci = [cil, cih];
end
