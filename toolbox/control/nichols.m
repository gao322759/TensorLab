function [mag, phase, wout] = nichols(sys, w)
%NICHOLS Nichols chart data of a SISO TF model.
%   [MAG,PHASE,W] = NICHOLS(SYS) evaluates the frequency response on a
%   default logarithmic grid and returns the magnitude in absolute units and
%   the phase in degrees (MATLAB's output convention).  Plotting is left to
%   the caller.
    arguments
        sys
        w = []
    end
    if isempty(w)
        w = logspace(-2, 2, 300);
    end
    h = freqresp(sys, w);
    mag = abs(h);
    phase = angle(h) * 180 / pi;
    wout = w;
end
