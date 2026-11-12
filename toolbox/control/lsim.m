function [y, tout] = lsim(sys, u, t, x0)
%LSIM Simulate the time response of a SISO TF model to an arbitrary input.
%   [Y,T] = LSIM(SYS,U,T) with input samples U at times T.
%   [Y,T] = LSIM(SYS,U,T,X0) is accepted for signature compatibility; for
%   transfer-function models the initial state is ignored (MATLAB behaviour).
%
%   The model is converted to state space (TF2SS) and integrated with a
%   fixed-step RK4 scheme; the input is linearly interpolated between
%   samples, matching MATLAB's default.
    arguments
        sys
        u
        t
        x0 = []
    end
    [num, den] = tfdata(sys);
    [A, B, C, D] = tf2ss(num, den);
    n = size(A, 1);
    x = zeros(n, 1);
    tt = t(:);
    N = numel(tt);
    nu = numel(u);
    if nu ~= 1 && nu ~= N
        error('lsim:inputSize', 'U must have one element per time sample');
    end
    y = zeros(N, 1);
    for k = 1:N
        if nu == 1
            uk = u;
            uk1 = u;
        else
            uk = u(k);
            if k < N
                uk1 = u(k + 1);
            else
                uk1 = uk;
            end
        end
        yk = C * x + D * uk;
        y(k) = yk(1);
        if k < N
            dt = tt(k + 1) - tt(k);
            umid = 0.5 * (uk + uk1);
            k1 = A * x + B * uk;
            k2 = A * (x + 0.5 * dt * k1) + B * umid;
            k3 = A * (x + 0.5 * dt * k2) + B * umid;
            k4 = A * (x + dt * k3) + B * uk1;
            x = x + (dt / 6) * (k1 + 2 * k2 + 2 * k3 + k4);
        end
    end
    tout = tt;
end
