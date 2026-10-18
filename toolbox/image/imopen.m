function B = imopen(A, SE)
%IMOPEN Morphological opening (erosion followed by dilation).
%   B = IMOPEN(A,SE) removes structures smaller than SE.
    arguments
        A
        SE
    end
    B = imdilate(imerode(A, SE), SE);
end
