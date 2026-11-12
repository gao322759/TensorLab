function y = dct(x, n)
%DCT Discrete cosine transform (DCT-II, orthonormal), matching MATLAB.
%   Y = DCT(X)   returns the DCT of X.
%   Y = DCT(X,N) pads or truncates X to length N first.
%
%   Y(k) = w(k) * sum_n x(n) * cos(pi*(2n-1)*(k-1)/(2N)),
%   with w(1) = sqrt(1/N), w(k) = sqrt(2/N) for k > 1.
    arguments
        x
        n = []
    end
    rowIn = 0;
    if size(x, 1) == 1 && numel(x) > 1
        rowIn = 1;
    end
    x = reshape(x, numel(x), 1);
    if ~isempty(n)
        if n < numel(x)
            x = x(1:n);
        elseif n > numel(x)
            x = [x; zeros(n - numel(x), 1)];
        end
        % 重新确保列向量（索引列向量可能返回行向量）
        x = reshape(x, numel(x), 1);
    end
    N = numel(x);
    k = (0:N-1)';
    m = (1:N)';
    M = cos((pi / (2 * N)) * (k * (2 * m - 1)'));
    w = sqrt(2 / N) * ones(N, 1);
    w(1) = sqrt(1 / N);
    y = w .* (M * x);
    if rowIn
        y = reshape(y, 1, numel(y));
    end
end
