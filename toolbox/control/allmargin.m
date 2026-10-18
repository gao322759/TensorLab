function [Gm, Pm, Wcg, Wcp] = allmargin(sys)
%ALLMARGIN All gain and phase margins of a SISO TF model.
%   [GM,PM,WCG,WCP] = ALLMARGIN(SYS) returns vectors with every stability
%   margin found on a dense logarithmic grid. MATLAB returns a struct here;
%   this kernel build returns numeric vectors.
    arguments
        sys
    end
    w = logspace(-3, 4, 3000);
    h = freqresp(sys, w);
    mag = abs(h);
    ph = angle(h) * 180 / pi;

    Gm = [];
    Wcg = [];
    for k = 2:numel(w)
        a = ph(k - 1) + 180;
        b = ph(k) + 180;
        if a * b <= 0
            t = -a / (b - a);
            mgc = mag(k - 1) + t * (mag(k) - mag(k - 1));
            Gm = [Gm; 1 / mgc];
            Wcg = [Wcg; w(k - 1) + t * (w(k) - w(k - 1))];
        end
    end

    Pm = [];
    Wcp = [];
    for k = 2:numel(w)
        a = mag(k - 1) - 1;
        b = mag(k) - 1;
        if a * b <= 0
            t = -a / (b - a);
            phc = ph(k - 1) + t * (ph(k) - ph(k - 1));
            Pm = [Pm; 180 + phc];
            Wcp = [Wcp; w(k - 1) + t * (w(k) - w(k - 1))];
        end
    end

    if isempty(Gm)
        Gm = Inf;
        Wcg = 0;
    end
    if isempty(Pm)
        Pm = Inf;
        Wcp = 0;
    end
end
