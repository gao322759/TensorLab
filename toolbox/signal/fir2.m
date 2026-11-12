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
    if n < 1
        error('fir2:invalidOrder', 'the filter order must be positive');
    end
    nfft = 512;
    while nfft < 4 * (n + 1)
        nfft = nfft * 2;
    end
    % 频点 k 对应 Nyquist 归一化频率 2k/nfft；用共轭对称构造实谱。
    H = zeros(1, nfft);
    half = nfft / 2;
    for k = 0:half
        fk = 2 * k / nfft;
        if fk > 1
            fk = 1;
        end
        H(k + 1) = tl_mag(f, m, fk);
    end
    for k = (half + 1):(nfft - 1)
        H(k + 1) = H(nfft - k + 1);
    end
    ht = real(ifft(H));
    w = hamming(n + 1);
    b = zeros(1, n + 1);
    if n == 2 * floor(n / 2)
        % 偶阶：把冲激响应循环移位到中心，得到线性相位（对称）系数
        halfn = n / 2;
        for i = 1:halfn
            b(i) = ht(nfft - halfn + i);
        end
        for i = 1:(halfn + 1)
            b(halfn + i) = ht(i);
        end
    else
        for i = 1:n + 1
            b(i) = ht(i);
        end
    end
    for i = 1:n + 1
        b(i) = b(i) * w(i);
    end
end

function v = tl_mag(f, m, x)
%TL_MAG Piecewise-linear magnitude with constant extrapolation.
    nf = numel(f);
    if x <= f(1)
        v = m(1);
        return
    end
    if x >= f(nf)
        v = m(nf);
        return
    end
    v = 0;
    for i = 1:(nf - 1)
        if x >= f(i) && x <= f(i + 1)
            d = f(i + 1) - f(i);
            if d == 0
                v = m(i + 1);
            else
                t = (x - f(i)) / d;
                v = m(i) + t * (m(i + 1) - m(i));
            end
            return
        end
    end
end
