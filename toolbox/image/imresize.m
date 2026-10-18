function B = imresize(A, scale, method)
%IMRESIZE Resize an image.
%   B = IMRESIZE(A, SCALE) resizes A by SCALE (scalar) or to SCALE(1)xSCALE(2).
%   B = IMRESIZE(A, SCALE, METHOD) with METHOD 'nearest' or 'bilinear'.
%   ('bicubic' falls back to bilinear.)  Pixel-centre mapping as MATLAB.
    arguments
        A
        scale
        method = 'nearest'
    end
    sz = size(A);
    m = sz(1);
    n = sz(2);
    p = size(A, 3);
    if numel(scale) == 1
        mr = round(m * scale);
        nr = round(n * scale);
    else
        mr = round(scale(1));
        nr = round(scale(2));
    end
    if mr < 1
        mr = 1;
    end
    if nr < 1
        nr = 1;
    end
    linear = 0;
    if strcmpi(method, 'bilinear')
        linear = 1;
    end
    if strcmpi(method, 'bicubic')
        linear = 1;
    end
    if strcmpi(method, 'linear')
        linear = 1;
    end
    B = zeros(mr, nr, p);
    for c = 1:p
        P = A(:, :, c);
        for i = 1:mr
            sr = (i - 0.5) * (m / mr) + 0.5;
            if linear
                i0 = floor(sr);
                di = sr - i0;
                if m < 2
                    i0 = 1;
                    di = 0;
                end
                if i0 < 1
                    i0 = 1;
                end
                if i0 > m - 1
                    i0 = m - 1;
                    di = sr - i0;
                end
                if i0 < 1
                    i0 = 1;
                end
                i1 = i0 + 1;
                if i1 > m
                    i1 = m;
                end
                if m < 2
                    i1 = 1;
                end
            else
                i0 = round(sr);
                if i0 < 1
                    i0 = 1;
                end
                if i0 > m
                    i0 = m;
                end
                i1 = i0;
                di = 0;
            end
            for j = 1:nr
                sc = (j - 0.5) * (n / nr) + 0.5;
                if linear
                    j0 = floor(sc);
                    dj = sc - j0;
                    if n < 2
                        j0 = 1;
                        dj = 0;
                    end
                    if j0 < 1
                        j0 = 1;
                    end
                    if j0 > n - 1
                        j0 = n - 1;
                        dj = sc - j0;
                    end
                    if j0 < 1
                        j0 = 1;
                    end
                    j1 = j0 + 1;
                    if j1 > n
                        j1 = n;
                    end
                    if n < 2
                        j1 = 1;
                    end
                    B(i, j, c) = P(i0, j0) * (1 - di) * (1 - dj) ...
                               + P(i0, j1) * (1 - di) * dj ...
                               + P(i1, j0) * di * (1 - dj) ...
                               + P(i1, j1) * di * dj;
                else
                    j0 = round(sc);
                    if j0 < 1
                        j0 = 1;
                    end
                    if j0 > n
                        j0 = n;
                    end
                    B(i, j, c) = P(i0, j0);
                end
            end
        end
    end
end
