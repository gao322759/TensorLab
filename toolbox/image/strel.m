function SE = strel(shape, params)
%STREL Structuring element for morphological operations.
%   SE = STREL('square',N)      N-by-N square (default N=3).
%   SE = STREL('rectangle',[M N])
%   SE = STREL('disk',R)        Euclidean disk of radius R.
%   SE = STREL('diamond',R)     Manhattan diamond.
%   SE = STREL('line',LEN,ANG)  line of length LEN at ANG degrees.
%   SE = STREL('octagon',N)     octagon approximated by a disk of radius N.
%
%   Returns a struct with fields Neighborhood and Dimensionality, matching
%   MATLAB's strel properties (subset). IMDILATE/IMERODE accept both this
%   struct and a numeric neighbourhood matrix.
    arguments
        shape = 'square'
        params = 3
    end
    N = 1;
    m = 1;
    n = 1;
    if strcmp(shape, 'square')
        if numel(params) == 1
            m = params;
            n = params;
        else
            m = params(1);
            n = params(2);
        end
        N = ones(m, n);
    elseif strcmp(shape, 'rectangle')
        m = params(1);
        n = params(2);
        N = ones(m, n);
    elseif strcmp(shape, 'disk') || strcmp(shape, 'octagon')
        r = params(1);
        d = 2 * r + 1;
        c = r + 1;
        N = zeros(d, d);
        for i = 1:d
            for j = 1:d
                if (i - c) ^ 2 + (j - c) ^ 2 <= r * r
                    N(i, j) = 1;
                end
            end
        end
    elseif strcmp(shape, 'diamond')
        r = params(1);
        d = 2 * r + 1;
        c = r + 1;
        N = zeros(d, d);
        for i = 1:d
            for j = 1:d
                if abs(i - c) + abs(j - c) <= r
                    N(i, j) = 1;
                end
            end
        end
    elseif strcmp(shape, 'line')
        len = params(1);
        if numel(params) > 1
            ang = params(2);
        else
            ang = 0;
        end
        if len < 1
            error('strel:line', 'line length must be >= 1');
        end
        d = len;
        c = floor(len / 2) + 1;
        N = zeros(d, d);
        th = ang * pi / 180;
        for t = -(len - 1) / 2:(len - 1) / 2
            x = round(t * cos(th));
            y = round(t * sin(th));
            ii = c + y;
            jj = c + x;
            if ii >= 1 && ii <= d && jj >= 1 && jj <= d
                N(ii, jj) = 1;
            end
        end
        if sum(sum(N)) == 0
            N(c, c) = 1;
        end
    else
        error('strel:shape', ['unsupported structuring element shape: ' shape]);
    end
    SE = struct('Neighborhood', N, 'Dimensionality', 2);
end
