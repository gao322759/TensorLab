function ycbcr = rgb2ycbcr(rgb)
%RGB2YCBCR Convert RGB to YCbCr (ITU-R BT.601, studio swing).
%   YCBCR = RGB2YCBCR(RGB) converts an M-by-N-by-3 double RGB image in the
%   range [0,1].  Matching MATLAB, the output uses the 8-bit studio-swing
%   range: Y in [16/255,235/255], Cb/Cr in [16/255,240/255].
    arguments
        rgb
    end
    if isempty(rgb) || size(rgb, 3) ~= 3
        error('rgb2ycbcr:invalidInput', 'RGB must be an M-by-N-by-3 image');
    end
    r = rgb(:, :, 1);
    g = rgb(:, :, 2);
    b = rgb(:, :, 3);
    ys = 0.299 * r + 0.587 * g + 0.114 * b;
    cbs = -0.168736 * r - 0.331264 * g + 0.5 * b;
    crs = 0.5 * r - 0.418688 * g - 0.081312 * b;
    y = 16 / 255 + (219 / 255) * ys;
    cb = 128 / 255 + (224 / 255) * cbs;
    cr = 128 / 255 + (224 / 255) * crs;
    ycbcr = cat(3, y, cb, cr);
end
