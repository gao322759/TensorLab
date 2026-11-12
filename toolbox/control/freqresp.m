function h = freqresp(sys, w)
%FREQRESP Frequency response of a SISO TF model.
%   H = FREQRESP(SYS,W) evaluates SYS at s = j*w for each frequency in W.
%   Matching MATLAB, H is 1-by-1-by-NUMEL(W).
    arguments
        sys
        w
    end
    num = sys.Numerator;
    den = sys.Denominator;
    j = sqrt(-1);
    nw = numel(w);
    h = zeros(1, nw);
    for k = 1:nw
        h(k) = polyval(num, j * w(k)) / polyval(den, j * w(k));
    end
    h = reshape(h, 1, 1, nw);
end
