function B = imcrop(A, rect)
%IMCROP Crop an image.
%   B = IMCROP(A,RECT) crops A to RECT = [xmin ymin width height]
%   (x = column, y = row; 1-based, matching MATLAB's non-interactive form).
%   B = IMCROP(A) returns A unchanged.
    arguments
        A
        rect = []
    end
    if isempty(rect)
        B = A;
        return
    end
    sz = size(A);
    x1 = round(rect(1));
    y1 = round(rect(2));
    w = round(rect(3));
    h = round(rect(4));
    if x1 < 1
        x1 = 1;
    end
    if y1 < 1
        y1 = 1;
    end
    if w < 1
        w = 1;
    end
    if h < 1
        h = 1;
    end
    x2 = x1 + w - 1;
    y2 = y1 + h - 1;
    if x2 > sz(2)
        x2 = sz(2);
    end
    if y2 > sz(1)
        y2 = sz(1);
    end
    B = A(y1:y2, x1:x2);
end
