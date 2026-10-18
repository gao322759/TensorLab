function h = freqresp(sys, w)
%FREQRESP Frequency response of a SISO TF model.
%   H = FREQRESP(SYS,W) evaluates SYS at s = j*w for each frequency in W.
    arguments
        sys
        w
    end
    num = sys.Numerator;
    den = sys.Denominator;
    j = sqrt(-1);
    h = zeros(1, numel(w));
    for k = 1:numel(w)
        h(k) = polyval(num, j * w(k)) / polyval(den, j * w(k));
    end
end
