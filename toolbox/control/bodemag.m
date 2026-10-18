function [mag, wout] = bodemag(sys, w)
%BODEMAG Bode magnitude only.
%   [MAG,W] = BODEMAG(SYS) or BODEMAG(SYS,W) returns the absolute gain.
    arguments
        sys
        w = []
    end
    if isempty(w)
        w = logspace(-2, 2, 200);
    end
    h = freqresp(sys, w);
    mag = abs(h);
    wout = w;
end
