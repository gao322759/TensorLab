function ycbcr = rgb2ycbcr(rgb)
%RGB2YCBCR Convert RGB to YCbCr (ITU-R BT.601).
%   YCBCR = RGB2YCBCR(RGB) converts an M-by-N-by-3 double RGB image in the
%   range [0,1] to YCbCr, matching MATLAB's coefficient set.
    arguments
        rgb
    end
    r = rgb(:, :, 1);
    g = rgb(:, :, 2);
    b = rgb(:, :, 3);
    y = 0.299 * r + 0.587 * g + 0.114 * b;
    cb = -0.168736 * r - 0.331264 * g + 0.5 * b + 0.5;
    cr = 0.5 * r - 0.418688 * g - 0.081312 * b + 0.5;
    ycbcr = cat(3, y, cb, cr);
end
