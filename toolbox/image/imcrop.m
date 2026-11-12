function B = imcrop(A, rect)
%IMCROP Crop an image.
%   B = IMCROP(A,RECT) crops A to RECT = [xmin ymin width height]
%   (x = column, y = row).  Following MATLAB's spatial convention, the
%   output contains every pixel whose centre lies inside the rectangle.
%   B = IMCROP(A) returns A unchanged.
    arguments
        A
        rect = []
    end
    if isempty(rect)
        B = A;
        return
    end
    if numel(rect) < 4
        error('imcrop:invalidRect', 'RECT must be [xmin ymin width height]');
    end
    sz = size(A);
    x1 = ceil(rect(1));
    y1 = ceil(rect(2));
    x2 = floor(rect(1) + rect(3));
    y2 = floor(rect(2) + rect(4));
    if x1 < 1
        x1 = 1;
    end
    if y1 < 1
        y1 = 1;
    end
    if x2 > sz(2)
        x2 = sz(2);
    end
    if y2 > sz(1)
        y2 = sz(1);
    end
    if x2 < x1
        x2 = x1;
    end
    if y2 < y1
        y2 = y1;
    end
    B = A(y1:y2, x1:x2);
end
