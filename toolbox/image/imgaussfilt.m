function B = imgaussfilt(A, sigma)
%IMGAUSSFILT 2-D Gaussian smoothing.
%   B = IMGAUSSFILT(A,SIGMA) filters A with a Gaussian kernel of the given
%   standard deviation.  Kernel size is 2*ceil(2*SIGMA)+1 and the borders
%   are replicated, matching MATLAB's default.
    arguments
        A
        sigma double = 0.5
    end
    if sigma <= 0
        error('imgaussfilt:expectedPositive', 'SIGMA must be a positive scalar');
    end
    sz = size(A);
    k = 2 * ceil(2 * sigma) + 1;
    h = fspecial('gaussian', k, sigma);
    hm = floor(k / 2);
    pages = size(A, 3);
    B = zeros(sz(1), sz(2), pages);
    for c = 1:pages
        P = A(:, :, c);
        for i = 1:sz(1)
            for j = 1:sz(2)
                s = 0;
                for a = 1:k
                    ii = i + a - 1 - hm;
                    if ii < 1
                        ii = 1;
                    end
                    if ii > sz(1)
                        ii = sz(1);
                    end
                    for b = 1:k
                        jj = j + b - 1 - hm;
                        if jj < 1
                            jj = 1;
                        end
                        if jj > sz(2)
                            jj = sz(2);
                        end
                        s = s + h(a, b) * P(ii, jj);
                    end
                end
                B(i, j, c) = s;
            end
        end
    end
end
