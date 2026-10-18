function stats = regionprops(L, props)
%REGIONPROPS Properties of connected regions.
%   STATS = REGIONPROPS(BW) returns Area, Centroid and BoundingBox for each
%   8-connected region of the logical image BW.
%   STATS = REGIONPROPS(L,PROPS) accepts a label matrix and a property name
%   (char or cell of chars).
%
%   Supported properties: Area, Centroid, BoundingBox, PixelIdxList,
%   MajorAxisLength, MinorAxisLength, Orientation, Perimeter.
%   MATLAB returns only the requested fields; this implementation returns a
%   fixed superset for non-'basic' requests (documented deviation).
    arguments
        L
        props = 'basic'
    end
    if strcmp(class(L), 'logical')
        [lab, n] = bwlabel(L, 8);
    else
        mx = max(max(L));
        if mx <= 1
            [lab, n] = bwlabel(L ~= 0, 8);
        else
            lab = L;
            n = mx;
        end
    end
    sz = size(L);
    m = sz(1);
    nc = sz(2);

    area = cell(1, n);
    cent = cell(1, n);
    bbox = cell(1, n);
    pix = cell(1, n);
    major = cell(1, n);
    minor = cell(1, n);
    orient = cell(1, n);
    perim = cell(1, n);

    for k = 1:n
        cnt = 0;
        sx = 0;
        sy = 0;
        xmin = nc + 1;
        xmax = 0;
        ymin = m + 1;
        ymax = 0;
        idxs = [];
        for j = 1:nc
            for i = 1:m
                if lab(i, j) == k
                    cnt = cnt + 1;
                    sx = sx + j;
                    sy = sy + i;
                    if j < xmin, xmin = j; end
                    if j > xmax, xmax = j; end
                    if i < ymin, ymin = i; end
                    if i > ymax, ymax = i; end
                    idxs = [idxs; (j - 1) * m + i];
                end
            end
        end
        if cnt == 0
            area{k} = 0;
            cent{k} = [0 0];
            bbox{k} = [0 0 0 0];
            pix{k} = [];
            major{k} = 0;
            minor{k} = 0;
            orient{k} = 0;
            perim{k} = 0;
        else
            cx = sx / cnt;
            cy = sy / cnt;
            mu20 = 0;
            mu02 = 0;
            mu11 = 0;
            boundary = 0;
            for j = 1:nc
                for i = 1:m
                    if lab(i, j) == k
                        dx = j - cx;
                        dy = i - cy;
                        mu20 = mu20 + dx * dx;
                        mu02 = mu02 + dy * dy;
                        mu11 = mu11 + dx * dy;
                        edge = 0;
                        if i == 1 || lab(i - 1, j) ~= k, edge = 1; end
                        if i == m || lab(i + 1, j) ~= k, edge = 1; end
                        if j == 1 || lab(i, j - 1) ~= k, edge = 1; end
                        if j == nc || lab(i, j + 1) ~= k, edge = 1; end
                        if edge == 1, boundary = boundary + 1; end
                    end
                end
            end
            mu20 = mu20 / cnt;
            mu02 = mu02 / cnt;
            mu11 = mu11 / cnt;
            common = sqrt((mu20 - mu02) ^ 2 + 4 * mu11 ^ 2);
            l1 = (mu20 + mu02 + common) / 2;
            l2 = (mu20 + mu02 - common) / 2;
            if l2 < 0, l2 = 0; end
            area{k} = cnt;
            cent{k} = [cx cy];
            bbox{k} = [xmin - 0.5, ymin - 0.5, xmax - xmin + 1, ymax - ymin + 1];
            pix{k} = idxs;
            major{k} = 4 * sqrt(l1);
            minor{k} = 4 * sqrt(l2);
            orient{k} = atan2(2 * mu11, mu20 - mu02) * 90 / pi;
            perim{k} = boundary;
        end
    end

    if strcmp(class(props), 'char')
        if strcmp(props, 'basic')
            stats = struct('Area', area, 'Centroid', cent, 'BoundingBox', bbox);
        elseif strcmp(props, 'Area')
            stats = struct('Area', area);
        elseif strcmp(props, 'Centroid')
            stats = struct('Centroid', cent);
        elseif strcmp(props, 'BoundingBox')
            stats = struct('BoundingBox', bbox);
        elseif strcmp(props, 'PixelIdxList')
            stats = struct('PixelIdxList', pix);
        else
            stats = struct('Area', area, 'Centroid', cent, 'BoundingBox', bbox, ...
                'PixelIdxList', pix, 'MajorAxisLength', major, ...
                'MinorAxisLength', minor, 'Orientation', orient, 'Perimeter', perim);
        end
    else
        stats = struct('Area', area, 'Centroid', cent, 'BoundingBox', bbox, ...
            'PixelIdxList', pix, 'MajorAxisLength', major, ...
            'MinorAxisLength', minor, 'Orientation', orient, 'Perimeter', perim);
    end
end
