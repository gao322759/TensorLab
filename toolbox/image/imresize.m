function B = imresize(A, scale, method)
%IMRESIZE Resize an image.
%   B = IMRESIZE(A,SCALE) resizes A by SCALE (scalar) or to
%   SCALE(1)-by-SCALE(2) pixels with bicubic interpolation (MATLAB default).
%   B = IMRESIZE(A,SCALE,METHOD) with METHOD 'nearest', 'bilinear' or
%   'bicubic'.
%
%   The sampling grid, kernel widening for antialiasing and symmetric
%   border mirroring follow MATLAB's contributions algorithm: output pixel
%   i maps to u = i/scale + 0.5*(1-1/scale) in input space, P =
%   ceil(kernel_width)+2 taps are evaluated and row-normalised, and
%   out-of-range indices are mirrored with period 2*N.
    arguments
        A
        scale
        method = 'bicubic'
    end
    sz = size(A);
    m = sz(1);
    n = sz(2);
    p = size(A, 3);
    if numel(scale) == 1
        if scale <= 0
            error('imresize:invalidScale', 'SCALE must be a positive number');
        end
        mr = ceil(m * scale);
        nr = ceil(n * scale);
        sy = scale;
        sx = scale;
    else
        mr = round(scale(1));
        nr = round(scale(2));
        if mr < 1 || nr < 1
            error('imresize:invalidSize', 'output size must be positive');
        end
        sy = mr / m;
        sx = nr / n;
    end
    [wy, iy] = tl_contrib(m, mr, sy, method);
    [wx, ix] = tl_contrib(n, nr, sx, method);
    ny = size(wy, 2);
    nx = size(wx, 2);
    B = zeros(mr, nr, p);
    for c = 1:p
        P0 = A(:, :, c);
        T = zeros(mr, n);
        for i = 1:mr
            for j = 1:n
                s = 0;
                for a = 1:ny
                    s = s + wy(i, a) * P0(iy(i, a), j);
                end
                T(i, j) = s;
            end
        end
        for i = 1:mr
            for j = 1:nr
                s = 0;
                for a = 1:nx
                    s = s + wx(j, a) * T(i, ix(j, a));
                end
                B(i, j, c) = s;
            end
        end
    end
end

function [W, I] = tl_contrib(nin, nout, s, method)
%TL_CONTRIB Interpolation weights and mirrored indices for one axis.
    if strcmpi(method, 'nearest')
        kw = 1;
        aa = 0;
    elseif strcmpi(method, 'bilinear') || strcmpi(method, 'linear')
        kw = 2;
        aa = 1;
    else
        kw = 4;
        aa = 1;
    end
    if s < 1 && aa
        kw = kw / s;
    end
    P = ceil(kw) + 2;
    W = zeros(nout, P);
    I = zeros(nout, P);
    aux = zeros(1, 2 * nin);
    for k = 1:nin
        aux(k) = k;
        aux(2 * nin + 1 - k) = k;
    end
    for i = 1:nout
        u = i / s + 0.5 * (1 - 1 / s);
        left = floor(u - kw / 2);
        tot = 0;
        for a = 1:P
            if s < 1 && aa
                W(i, a) = s * tl_kernel((u - left - a + 1) * s, method);
            else
                W(i, a) = tl_kernel(u - left - a + 1, method);
            end
            tot = tot + W(i, a);
        end
        for a = 1:P
            if tot ~= 0
                W(i, a) = W(i, a) / tot;
            end
            I(i, a) = aux(mod(left + a - 2, 2 * nin) + 1);
        end
    end
end

function w = tl_kernel(x, method)
%TL_KERNEL Interpolation kernel value at x.
    if strcmpi(method, 'nearest')
        if x >= -0.5 && x < 0.5
            w = 1;
        else
            w = 0;
        end
    elseif strcmpi(method, 'bilinear') || strcmpi(method, 'linear')
        ax = abs(x);
        if ax <= 1
            w = 1 - ax;
        else
            w = 0;
        end
    else
        w = tl_cubic(x);
    end
end

function w = tl_cubic(x)
%TL_CUBIC Keys cubic interpolation kernel (a = -0.5).
    ax = abs(x);
    if ax <= 1
        w = (1.5 * ax - 2.5) * ax * ax + 1;
    elseif ax < 2
        w = ((-0.5 * ax + 2.5) * ax - 4) * ax + 2;
    else
        w = 0;
    end
end
