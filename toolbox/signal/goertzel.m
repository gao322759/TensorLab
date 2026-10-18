function y = goertzel(x, ind)
%GOERTZEL Discrete Fourier transform at selected bins.
%   Y = GOERTZEL(X,IND) evaluates the DFT of X at the 1-based frequency
%   indices IND.
    arguments
        x
        ind
    end
    x = x(:)';
    N = numel(x);
    if numel(ind) == 1
        k = ind(1) - 1;
        acc = 0;
        for n = 0:N - 1
            acc = acc + x(n + 1) * exp(-1i * 2 * pi * k * n / N);
        end
        y = acc;
    else
        y = zeros(1, numel(ind));
        for kk = 1:numel(ind)
            k = ind(kk) - 1;
            acc = 0;
            for n = 0:N - 1
                acc = acc + x(n + 1) * exp(-1i * 2 * pi * k * n / N);
            end
            y(kk) = acc;
        end
    end
end
