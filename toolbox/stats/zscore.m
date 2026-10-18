function z = zscore(x, flag)
%ZSCORE Standardized z-scores.
%   Z = ZSCORE(X)   centers to mean 0 and scales by the sample std (N-1).
%   Z = ZSCORE(X,1) scales by the population std (N).
    arguments
        x
        flag double = 0
    end
    mu = mean(x);
    if flag == 0
        s = std(x);
    else
        s = std(x, 1);
    end
    z = (x - mu) ./ s;
end
