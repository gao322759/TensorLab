function [Gm, Pm, Wcg, Wcp] = margin(sys)
%MARGIN Gain and phase margins of a SISO TF model.
%   [GM,PM,WCG,WCP] = MARGIN(SYS).
%   GM  gain margin (absolute, not dB), WCG its frequency (rad/s).
%   PM  phase margin in degrees, WCP its frequency (rad/s).
%   Computed numerically on a dense logarithmic grid with linear
%   interpolation, after Ogata, Modern Control Engineering, 5th ed., ch. 7.
    arguments
        sys
    end
    w = logspace(-3, 3, 4000);
    h = freqresp(sys, w);
    mag = abs(h);
    ph = angle(h) * 180 / pi;

    Wcp = 0;
    Pm = 0;
    for k = 2:numel(w)
        if (mag(k - 1) - 1) * (mag(k) - 1) <= 0
            Wcp = w(k);
            t = (1 - mag(k - 1)) / (mag(k) - mag(k - 1));
            phc = ph(k - 1) + t * (ph(k) - ph(k - 1));
            Pm = 180 + phc;
            break
        end
    end

    Wcg = 0;
    Gm = 1;
    for k = 2:numel(w)
        if (ph(k - 1) + 180) * (ph(k) + 180) <= 0
            Wcg = w(k);
            t = (-180 - ph(k - 1)) / (ph(k) - ph(k - 1));
            mgc = mag(k - 1) + t * (mag(k) - mag(k - 1));
            Gm = 1 / mgc;
            break
        end
    end
end
