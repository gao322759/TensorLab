function [y, tout] = lsim(sys, u, t, x0)
%LSIM Simulate the time response of a SISO TF model to an arbitrary input.
%   [Y,T] = LSIM(SYS,U,T) with input samples U at times T (zero initial state).
%   [Y,T] = LSIM(SYS,U,T,X0) with initial state X0.
%
%   The model is converted to state space (TF2SS) and integrated with a
%   fixed-step RK4 scheme using piecewise-constant inputs (documented subset;
%   MATLAB interpolates the input linearly).
    arguments
        sys
        u
        t
        x0 = []
    end
    [num, den] = tfdata(sys);
    [A, B, C, D] = tf2ss(num, den);
    n = size(A, 1);
    if isempty(x0)
        x = zeros(n, 1);
    elseif n == 1
        x = zeros(1, 1);
        x(1) = x0;
    else
        x = x0(:);
    end
    N = numel(t);
    y = zeros(N, 1);
    for k = 1:N
        uk = u(k);
        yk = C * x + D * uk;
        y(k) = yk(1);
        if k < N
            dt = t(k + 1) - t(k);
            k1 = A * x + B * uk;
            k2 = A * (x + 0.5 * dt * k1) + B * uk;
            k3 = A * (x + 0.5 * dt * k2) + B * uk;
            k4 = A * (x + dt * k3) + B * uk;
            x = x + (dt / 6) * (k1 + 2 * k2 + 2 * k3 + k4);
        end
    end
    tout = t;
end
