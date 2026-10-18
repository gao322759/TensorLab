function B = imgaussfilt(A, sigma)
%IMGAUSSFILT 2-D Gaussian smoothing.
%   B = IMGAUSSFILT(A,SIGMA) filters A with a Gaussian kernel of the given
%   standard deviation.  Kernel size is 2*ceil(2*SIGMA)+1.
    arguments
        A
        sigma double = 0.5
    end
    h = fspecial('gaussian', 2 * ceil(2 * sigma) + 1, sigma);
    B = imfilter(A, h);
end
