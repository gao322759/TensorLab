function b = fir2(n, f, m)
%FIR2 Frequency-sampling FIR filter design.
%   B = FIR2(N,F,M) designs an N-th order FIR filter whose magnitude follows
%   the piecewise-linear curve (F,M), with F in [0,1] (1 = Nyquist).
%
%   Method: dense frequency grid + inverse DFT + Hamming window
%   (Oppenheim & Schafer, Discrete-Time Signal Processing, 3rd ed., ch. 7).
    arguments
        n
        f
        m
    end
    nfft = 512;
    while nfft < 4 * n + 1
        nfft = nfft * 2;
    end
    grid = zeros(1, nfft + 1);
    for i = 0:nfft
        grid(i + 1) = i / nfft;
    end
    h = interp1(f, m, grid);
    for i = 1:nfft + 1
        if isnan(h(i))
            if grid(i) <= f(1)
                h(i) = m(1);
            else
                h(i) = m(numel(m));
            end
        end
    end

    H = zeros(1, 2 * nfft);
    for i = 1:nfft + 1
        H(i) = h(i);
    end
    for i = 2:nfft
        H(2 * nfft - i + 2) = h(i);
    end
    ht = real(ifft(H));
    w = hamming(n + 1);
    b = zeros(1, n + 1);
    for i = 1:n + 1
        b(i) = ht(i) * w(i);
    end
    % Normalize the first passband to its target gain.
    if f(1) == 0 && m(1) ~= 0
        g = 0;
        for i = 1:n + 1
            g = g + b(i);
        end
        if abs(g) > 1e-12
            b = b * (m(1) / g);
        end
    end
end
