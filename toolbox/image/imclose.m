function B = imclose(A, SE)
%IMCLOSE Morphological closing (dilation followed by erosion).
%   B = IMCLOSE(A,SE) fills gaps smaller than SE.  The input is padded with
%   zeros by the structuring-element radius before the two passes so that
%   the border is handled the same way as MATLAB.
    arguments
        A
        SE
    end
    if strcmp(class(SE), 'struct')
        N = SE.Neighborhood;
    else
        N = SE;
    end
    ss = size(N);
    pm = floor(ss(1) / 2);
    pn = floor(ss(2) / 2);
    sz = size(A);
    Ap = zeros(sz(1) + 2 * pm, sz(2) + 2 * pn);
    for i = 1:sz(1)
        for j = 1:sz(2)
            Ap(i + pm, j + pn) = A(i, j);
        end
    end
    Bp = imerode(imdilate(Ap, SE), SE);
    B = zeros(sz(1), sz(2));
    for i = 1:sz(1)
        for j = 1:sz(2)
            B(i, j) = Bp(i + pm, j + pn);
        end
    end
end
