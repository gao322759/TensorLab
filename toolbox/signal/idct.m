function x = idct(y, n)
%IDCT Inverse discrete cosine transform (inverse of DCT).
%   X = IDCT(Y)   returns the inverse DCT of Y.
%   X = IDCT(Y,N) pads or truncates Y to length N first.
    arguments
        y
        n = []
    end
    y = reshape(y, numel(y), 1);
    if ~isempty(n)
        if n < numel(y)
            y = y(1:n);
        elseif n > numel(y)
            y = [y; zeros(n - numel(y), 1)];
        end
    end
    N = numel(y);
    k = (0:N-1)';
    m = (1:N)';
    M = cos((pi / (2 * N)) * (k * (2 * m - 1)'));
    w = sqrt(2 / N) * ones(N, 1);
    w(1) = sqrt(1 / N);
    x = M' * (w .* y);
end
