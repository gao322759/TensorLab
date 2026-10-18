function BW = edge(I, method, thresh)
%EDGE Edge detection (subset of MATLAB's edge).
%   BW = EDGE(I)               Sobel gradient magnitude, automatic threshold.
%   BW = EDGE(I,METHOD)        METHOD in {'sobel','prewitt','roberts','laplacian','log'}.
%   BW = EDGE(I,METHOD,THRESH) explicit threshold (gradient methods).
%
%   Returns a double matrix of 0/1.  Canny is not implemented.
%   Reference: Gonzalez & Woods, Digital Image Processing, 4th ed., ch. 10.
    arguments
        I
        method = 'sobel'
        thresh = []
    end
    D = double(I);
    if strcmpi(method, 'sobel')
        hx = [-1 0 1; -2 0 2; -1 0 1];
        hy = hx';
    elseif strcmpi(method, 'prewitt')
        hx = [-1 0 1; -1 0 1; -1 0 1];
        hy = hx';
    elseif strcmpi(method, 'roberts')
        hx = [1 0; 0 -1];
        hy = [0 1; -1 0];
    elseif strcmpi(method, 'laplacian')
        hx = [0 -1 0; -1 4 -1; 0 -1 0];
        hy = zeros(1, 1);
    else
        % Laplacian of Gaussian (approx. via Gaussian smoothing + Laplacian)
        hg = fspecial('gaussian', 7, 1.0);
        D = imfilter(D, hg);
        hx = [0 -1 0; -1 4 -1; 0 -1 0];
        hy = zeros(1, 1);
    end

    if numel(hy) == 1 && hy(1) == 0
        mag = abs(imfilter(D, hx));
    else
        gx = imfilter(D, hx);
        gy = imfilter(D, hy);
        mag = sqrt(gx.^2 + gy.^2);
    end

    if isempty(thresh)
        thresh = 0.1 * max(max(mag));
    end
    BW = double(mag > thresh);
end
