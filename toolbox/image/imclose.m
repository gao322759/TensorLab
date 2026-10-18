function B = imclose(A, SE)
%IMCLOSE Morphological closing (dilation followed by erosion).
%   B = IMCLOSE(A,SE) fills gaps smaller than SE.
    arguments
        A
        SE
    end
    B = imerode(imdilate(A, SE), SE);
end
