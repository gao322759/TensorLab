function [coeff, score, latent, tsquared, explained] = pca(X)
%PCA Principal component analysis (subset of MATLAB's pca).
%   COEFF     eigenvectors of the covariance of the centered rows, descending.
%   SCORE     centered data projected onto COEFF.
%   LATENT    eigenvalues (variances) of the principal components.
%   TSQUARED  Hotelling's T-squared statistic for each row.
%   EXPLAINED percentage of total variance explained by each component.
%
%   Sign convention: the largest-magnitude entry of each COEFF column is
%   made positive, matching MATLAB.  Implemented via COV + EIG (Jolliffe,
%   Principal Component Analysis, 2nd ed., ch. 3).
    arguments
        X
    end
    n = size(X, 1);
    Xc = X - repmat(mean(X, 1), n, 1);
    C = cov(Xc);
    [V, D] = eig(C);
    d = diag(D);
    [latent, idx] = sort(d, 'descend');
    coeff = V(:, idx);
    for j = 1:size(coeff, 2)
        col = coeff(:, j);
        [mv, mi] = max(abs(col));
        if col(mi) < 0
            coeff(:, j) = -col;
        end
    end
    score = Xc * coeff;
    tsquared = sum((score.^2) ./ repmat(latent', n, 1), 2);
    explained = 100 * latent / sum(latent);
end
