function [y, tout] = initial(sys, x0, t)
%INITIAL Zero-input response of a SISO TF model.
%   [Y,T] = INITIAL(SYS,X0) simulates the free response from state X0.
%   [Y,T] = INITIAL(SYS,X0,T) uses the time vector T.
    arguments
        sys
        x0
        t = []
    end
    if isempty(t)
        t = linspace(0, 5, 101)';
    end
    u = zeros(numel(t), 1);
    [y, tout] = lsim(sys, u, t, x0);
end
