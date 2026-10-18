function [magdb, phase, wout] = nichols(sys, w)
%NICHOLS Nichols chart data of a SISO TF model.
%   [MAGDB,PHASE,W] = NICHOLS(SYS) evaluates the frequency response on a
%   default logarithmic grid and returns magnitude in dB and phase in
%   degrees. Plotting is left to the caller.
    arguments
        sys
        w = []
    end
    if isempty(w)
        w = logspace(-2, 2, 300);
    end
    h = freqresp(sys, w);
    magdb = 20 * log10(abs(h));
    phase = angle(h) * 180 / pi;
    wout = w;
end
