function BW = edge(I, method, thresh)
%EDGE Edge detection (subset of MATLAB's edge).
%   BW = EDGE(I)               Sobel gradient magnitude, automatic threshold.
%   BW = EDGE(I,METHOD)        METHOD in {'sobel','prewitt','roberts','log'}.
%   BW = EDGE(I,METHOD,THRESH) explicit threshold (gradient methods).
%
%   Sobel/Prewitt/Roberts follow MATLAB: gradients are computed with
%   replicated borders, the magnitude is b = bx^2 + by^2, the automatic
%   cutoff is scale*mean(b) (scale 4 for Sobel/Prewitt, 6 for Roberts) and
%   non-maximum suppression thins the ridge along the gradient direction.
%   Reference: Gonzalez & Woods, Digital Image Processing, 4th ed., ch. 10.
    arguments
        I
        method = 'sobel'
        thresh = []
    end
    D = double(I);
    if isempty(D)
        BW = zeros(size(D));
        return
    end
    thin = 0;
    h = [0 0 0; 0 0 0; 0 0 0];
    xm = zeros(2);
    ym = zeros(2);
    scale = 4;
    if strcmpi(method, 'sobel')
        h = [1 2 1; 0 0 0; -1 -2 -1] / 8;
        scale = 4;
        thin = 1;
    elseif strcmpi(method, 'prewitt')
        h = [1 1 1; 0 0 0; -1 -1 -1] / 6;
        scale = 4;
        thin = 1;
    elseif strcmpi(method, 'roberts')
        xm = [1 0; 0 -1] / 2;
        ym = [0 1; -1 0] / 2;
        scale = 6;
        thin = 1;
    elseif strcmpi(method, 'laplacian') || strcmpi(method, 'log')
        hl = [0 -1 0; -1 4 -1; 0 -1 0];
        if strcmpi(method, 'log')
            hg = fspecial('gaussian', 7, 1.0);
            D = imfilter(D, hg);
        end
        mag = abs(imfilter(D, hl));
        if isempty(thresh)
            thresh = 0.1 * max(max(mag));
        end
        BW = double(mag > thresh);
        return
    else
        error('edge:invalidInputString', 'Invalid input string: %s', method);
    end
    if strcmpi(method, 'roberts')
        bx = imfilter(D, xm, 'replicate');
        by = imfilter(D, ym, 'replicate');
    else
        bx = imfilter(D, h', 'replicate');
        by = imfilter(D, h, 'replicate');
    end
    b = bx .* bx + by .* by;
    if isempty(thresh)
        cutoff = scale * sum(sum(b)) / numel(b);
    else
        cutoff = thresh * thresh;
    end
    if thin
        BW = tl_nms(b, bx, by, cutoff);
    else
        BW = double(b > cutoff);
    end
end

function e = tl_nms(b, bx, by, cutoff)
%TL_NMS Non-maximum suppression along the quantised gradient direction.
    m = size(b, 1);
    n = size(b, 2);
    e = zeros(m, n);
    for i = 1:m
        for j = 1:n
            if b(i, j) > cutoff
                i1 = i;
                j1 = j;
                i2 = i;
                j2 = j;
                ang = atan2(by(i, j), bx(i, j)) * 180 / pi;
                while ang < 0
                    ang = ang + 180;
                end
                while ang >= 180
                    ang = ang - 180;
                end
                if ang < 22.5 || ang >= 157.5
                    i1 = i;
                    j1 = j - 1;
                    i2 = i;
                    j2 = j + 1;
                elseif ang < 67.5
                    i1 = i + 1;
                    j1 = j + 1;
                    i2 = i - 1;
                    j2 = j - 1;
                elseif ang < 112.5
                    i1 = i - 1;
                    j1 = j;
                    i2 = i + 1;
                    j2 = j;
                else
                    i1 = i + 1;
                    j1 = j - 1;
                    i2 = i - 1;
                    j2 = j + 1;
                end
                i1 = min(max(i1, 1), m);
                i2 = min(max(i2, 1), m);
                j1 = min(max(j1, 1), n);
                j2 = min(max(j2, 1), n);
                tol = 100 * eps;
                if b(i, j) + tol >= b(i1, j1) && b(i, j) + tol >= b(i2, j2)
                    e(i, j) = 1;
                end
            end
        end
    end
end
