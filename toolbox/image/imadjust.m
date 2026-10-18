function J = imadjust(I, in, out, gamma)
%IMADJUST Adjust image intensity values / contrast.
%   J = IMADJUST(I) maps the data range to [0,1].
%   J = IMADJUST(I,[LOW_IN HIGH_IN],[LOW_OUT HIGH_OUT],GAMMA).
    arguments
        I
        in = []
        out = []
        gamma double = 1
    end
    if isempty(in)
        in = [min(min(I)), max(max(I))];
    end
    if isempty(out)
        out = [0, 1];
    end
    lo = in(1);
    hi = in(2);
    sz = size(I);
    J = zeros(sz(1), sz(2));
    for i = 1:sz(1)
        for j = 1:sz(2)
            t = (I(i, j) - lo) / (hi - lo);
            if t < 0
                t = 0;
            end
            if t > 1
                t = 1;
            end
            if gamma ~= 1
                t = t^gamma;
            end
            J(i, j) = out(1) + (out(2) - out(1)) * t;
        end
    end
end
