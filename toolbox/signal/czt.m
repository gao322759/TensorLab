function y = czt(x, m, w, a)
%CZT Chirp Z-transform.
%   Y = CZT(X) returns the Z-transform of X at M points equally spaced on the
%   unit circle (M defaults to numel(X)).
%   Y = CZT(X,M,W,A) evaluates on the contour z = A*W.^-(0:M-1), i.e.
%   Y(k) = sum_n X(n) * z_k^(-(n-1)).
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
    y = zeros(1, m);
    for k = 1:m
        magz = maga * magw ^ (-(k - 1));
        argz = pha - phw * (k - 1);
        if magz == 0
            error('czt:contour', 'the contour passes through zero');
        end
        step = (1 / magz) * exp(-1i * argz);
        acc = 0;
        p = 1;
        for n = 1:N
            acc = acc + x(n) * p;
            p = p * step;
        end
        y(k) = acc;
    end
end
