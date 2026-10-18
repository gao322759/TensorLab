function y = czt(x, m, w, a)
%CZT Chirp Z-transform.
%   Y = CZT(X) returns the Z-transform of X at M points equally spaced on the
%   unit circle (M defaults to numel(X)).
%   Y = CZT(X,M,W,A) evaluates on the contour A*W.^-(0:M-1).
%
%   Direct summation (O(N*M)); sufficient for moderate lengths.
    arguments
        x
        m = []
        w = []
        a = []
    end
    x = x(:)';
    N = numel(x);
    if isempty(m)
        m = N;
    end
    if isempty(w)
        w = exp(-2i * pi / N);
    end
    if isempty(a)
        a = 1;
    end
    magw = abs(w);
    phw = angle(w);
    maga = abs(a);
    pha = angle(a);
    if m == 1
        acc = 0;
        for n = 1:N
            acc = acc + x(n) * maga ^ (-(n - 1)) * exp(-1i * pha * (n - 1));
        end
        y = acc;
    else
        y = zeros(1, m);
        for k = 1:m
            acc = 0;
            for n = 1:N
                acc = acc + x(n) * maga ^ (-(n - 1)) * exp(-1i * pha * (n - 1)) ...
                    * magw ^ (-(n - 1) * (k - 1)) * exp(-1i * phw * (n - 1) * (k - 1));
            end
            y(k) = acc;
        end
    end
end
