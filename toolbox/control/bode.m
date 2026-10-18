function [mag, phase, wout] = bode(sys, w)
%BODE Bode magnitude and phase of a SISO TF model (numeric form).
%   [MAG,PHASE,W] = BODE(SYS)    uses a default logarithmic grid.
%   [MAG,PHASE,W] = BODE(SYS,W)  evaluates on the frequencies W (rad/s).
%   MAG is absolute gain, PHASE is degrees.
    arguments
        sys
        w = []
    end
    if isempty(w)
        w = logspace(-2, 2, 200);
    end
    h = freqresp(sys, w);
    mag = abs(h);
    phase = angle(h) * 180 / pi;
    wout = w;
end
