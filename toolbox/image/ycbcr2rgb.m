function rgb = ycbcr2rgb(ycbcr)
%YCBCR2RGB Convert YCbCr to RGB (ITU-R BT.601).
%   RGB = YCBCR2RGB(YCBCR) is the inverse of RGB2YCBCR.
    arguments
        ycbcr
    end
    y = ycbcr(:, :, 1);
    cb = ycbcr(:, :, 2) - 0.5;
    cr = ycbcr(:, :, 3) - 0.5;
    r = y + 1.402 * cr;
    g = y - 0.344136 * cb - 0.714136 * cr;
    b = y + 1.772 * cb;
    rgb = cat(3, r, g, b);
end
