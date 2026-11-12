function [coeff, score, latent, tsquared, explained] = pca(X)
%PCA Principal component analysis (subset of MATLAB's pca).
%   COEFF     eigenvectors of the covariance of the centered rows, descending.
%   SCORE     centered data projected onto COEFF.
%   LATENT    eigenvalues (variances) of the principal components.
%   TSQUARED  Hotelling's T-squared statistic for each row.
%   EXPLAINED percentage of total variance explained by each component.
%
%   Matching MATLAB, at most min(N-1,P) components are returned (N rows,
%   P variables).  Sign convention: the largest-magnitude entry of each
%   COEFF column is made positive.  Implemented via COV + EIG (Jolliffe,
%   Principal Component Analysis, 2nd ed., ch. 3).
    arguments
        X
    end
    n = size(X, 1);
    p = size(X, 2);
    if n < 2
        error('pca:tooFewObservations', 'X must have at least two rows');
    end
    Xc = X - repmat(mean(X, 1), n, 1);
    C = cov(Xc);
    [V, D] = eig(C);
    d = diag(D);
    [ds, idx] = sort(d, 'descend');
    total = sum(ds);
    k = n - 1;
    if p < k
        k = p;
    end
    latent = zeros(k, 1);
    coeff = zeros(p, k);
    for j = 1:k
        latent(j) = ds(j);
        for r = 1:p
            coeff(r, j) = V(r, idx(j));
        end
    end
    for j = 1:k
        [mv, mi] = max(abs(coeff(:, j)));
        if coeff(mi, j) < 0
            for r = 1:p
                coeff(r, j) = -coeff(r, j);
            end
        end
    end
    score = Xc * coeff;
    tsquared = sum((score.^2) ./ repmat(latent', n, 1), 2);
    if total > 0
        explained = 100 * latent / total;
    else
        explained = zeros(k, 1);
    end
end
