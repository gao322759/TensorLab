function y = decimate(x, r)
%DECIMATE Decimate by an integer factor with an anti-aliasing prefilter.
%   Y = DECIMATE(X,R) lowpass-filters X and keeps every R-th sample.
%
%   Matching MATLAB's default, the anti-aliasing filter is an 8th-order
%   Chebyshev Type I lowpass with 0.05 dB passband ripple and cutoff 0.8/R,
%   applied zero-phase (filtfilt, odd padding + steady-state initial
%   conditions).  See Oppenheim & Schafer, ch. 7.
    arguments
        x
        r double
    end
    if r < 1 || r ~= round(r)
        error('decimate:invalidFactor', 'R must be a positive integer');
    end
    rowIn = 0;
    if size(x, 1) == 1 && numel(x) > 1
        rowIn = 1;
    end
    x = reshape(x, numel(x), 1);
    % MATLAB 会在 8 阶滤波器不满足 -0.05 dB 通带边缘容差时逐阶降阶
    wn = 0.8 / r;
    nfilt = 8;
    [b, a] = tl_cheby1(nfilt, 0.05, wn);
    while tl_all_zero(b) || abs(tl_mag_db(b, a, wn) + 0.05) > 1e-6
        nfilt = nfilt - 1;
        if nfilt < 1
            break
        end
        [b, a] = tl_cheby1(nfilt, 0.05, wn);
    end
    nf = numel(a);
    if numel(x) > 3 * (nf - 1)
        xf = tl_filtfilt(b, a, x);
    else
        xf = tl_filter(tl_pad(b, nf), a, x, zeros(nf - 1, 1));
    end
    nd = numel(x);
    nout = ceil(nd / r);
    nbeg = r - (r * nout - nd);
    y = xf(nbeg:r:nd);
    y = reshape(y, numel(y), 1);
    if rowIn
        y = reshape(y, 1, numel(y));
    end
end

function tf = tl_all_zero(v)
%TL_ALL_ZERO True when every element of v is zero.
    tf = 1;
    for i = 1:numel(v)
        if v(i) ~= 0
            tf = 0;
        end
    end
end

function m = tl_mag_db(b, a, f)
%TL_MAG_DB Magnitude response of H(z) in dB at frequency f (1 = Nyquist).
    x = exp(-1i * pi * f);
    num = 0;
    for k = 1:numel(b)
        num = num + b(k) * x ^ (k - 1);
    end
    den = 0;
    for k = 1:numel(a)
        den = den + a(k) * x ^ (k - 1);
    end
    m = 20 * log10(abs(num / den));
end

function [b, a] = tl_cheby1(n, rp, wn)
%TL_CHEBY1 Chebyshev Type I digital lowpass (bilinear transform).
    if wn <= 0 || wn >= 1
        error('decimate:invalidCutoff', 'cutoff must be in (0,1)');
    end
    eps = sqrt(10 ^ (rp / 10) - 1);
    alpha = asinh(1 / eps) / n;
    sh = sinh(alpha);
    ch = cosh(alpha);
    om = tan(pi * wn / 2);
    a = 1;
    for k = 1:n
        th = pi * (2 * k - 1) / (2 * n);
        p = om * (-sh * sin(th) + 1i * ch * cos(th));
        z = (1 + p) / (1 - p);
        a = conv(a, [1, -z]);
    end
    a = real(a);
    g = sum(a) / 2 ^ n;
    if mod(n, 2) == 0
        % 偶阶 Chebyshev I 的直流增益为 1/sqrt(1+eps^2)
        g = g / sqrt(1 + eps * eps);
    end
    bin = 1;
    for i = 1:n
        bin = conv(bin, [1, 1]);
    end
    b = g * bin;
end

function bp = tl_pad(b, n)
%TL_PAD Zero-pad a numerator/denominator to length n.
    bp = zeros(1, n);
    for i = 1:numel(b)
        bp(i) = b(i);
    end
end

function zi = tl_zi(b, a, u)
%TL_ZI Steady-state DF2T state for a constant input u.
    n = numel(a);
    g = sum(b) / sum(a);
    y = g * u;
    zi = zeros(n - 1, 1);
    for i = (n - 1):-1:1
        zi(i) = b(i + 1) * u - a(i + 1) * y;
        if i < (n - 1)
            zi(i) = zi(i) + zi(i + 1);
        end
    end
end

function [y, zf] = tl_filter(b, a, x, zi)
%TL_FILTER Direct-form II transposed filter.
    n = numel(a);
    N = numel(x);
    z = zi(:);
    y = zeros(N, 1);
    for k = 1:N
        xn = x(k);
        yk = b(1) * xn;
        if n > 1
            yk = yk + z(1);
        end
        y(k) = yk;
        if n > 1
            for i = 1:(n - 2)
                z(i) = b(i + 1) * xn + z(i + 1) - a(i + 1) * yk;
            end
            z(n - 1) = b(n) * xn - a(n) * yk;
        end
    end
    zf = z;
end

function y = tl_filtfilt(b, a, x)
%TL_FILTFILT Zero-phase forward-backward filtering with odd padding.
    nf = max(numel(b), numel(a));
    bp = tl_pad(b, nf);
    ap = tl_pad(a, nf);
    N = numel(x);
    padlen = 3 * (nf - 1);
    xp = zeros(N + 2 * padlen, 1);
    for i = 1:padlen
        xp(i) = 2 * x(1) - x(padlen + 2 - i);
    end
    for i = 1:N
        xp(padlen + i) = x(i);
    end
    for i = 1:padlen
        xp(padlen + N + i) = 2 * x(N) - x(N - i);
    end
    zi = tl_zi(bp, ap, xp(1));
    yf = tl_filter(bp, ap, xp, zi);
    xr = flipud(yf);
    zi2 = tl_zi(bp, ap, xr(1));
    yr = tl_filter(bp, ap, xr, zi2);
    yfull = flipud(yr);
    y = zeros(N, 1);
    for i = 1:N
        y(i) = yfull(padlen + i);
    end
end
