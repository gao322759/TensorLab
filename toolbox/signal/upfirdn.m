function y = upfirdn(x, h, p, q)
%UPFIRDN Upsample, FIR filter, downsample.
%   Y = UPFIRDN(X,H,P,Q) upsamples X by P, filters with H, and downsamples
%   by Q. Length(Y) = ceil(((numel(X)-1)*P + numel(H))/Q).
    arguments
        x
        h
        p double = 1
        q double = 1
    end
    x = x(:)';
    h = h(:)';
    lx = numel(x);
    lh = numel(h);
    lu = (lx - 1) * p + 1;
    xu = zeros(1, lu);
    for i = 1:lx
        xu((i - 1) * p + 1) = x(i);
    end
    yf = conv(xu, h);
    ly = ceil(((lx - 1) * p + lh) / q);
    y = zeros(1, ly);
    for i = 1:ly
        idx = (i - 1) * q + 1;
        if idx <= numel(yf)
            y(i) = yf(idx);
        end
    end
end
