function rgb = ycbcr2rgb(ycbcr)
%YCBCR2RGB Convert YCbCr to RGB (ITU-R BT.601, studio swing).
%   RGB = YCBCR2RGB(YCBCR) is the inverse of RGB2YCBCR.  Matching MATLAB,
%   the studio-swing offsets are removed and the result is clipped to [0,1].
    arguments
        ycbcr
    end
    if isempty(ycbcr) || size(ycbcr, 3) ~= 3
        error('ycbcr2rgb:invalidInput', 'YCBCR must be an M-by-N-by-3 image');
    end
    y = (ycbcr(:, :, 1) - 16 / 255) * (255 / 219);
    cb = (ycbcr(:, :, 2) - 128 / 255) * (255 / 224);
    cr = (ycbcr(:, :, 3) - 128 / 255) * (255 / 224);
    r = y + 1.402 * cr;
    g = y - 0.344136 * cb - 0.714136 * cr;
    b = y + 1.772 * cb;
    rgb = cat(3, r, g, b);
    for i = 1:numel(rgb)
        if rgb(i) < 0
            rgb(i) = 0;
        elseif rgb(i) > 1
            rgb(i) = 1;
        end
    end
end
