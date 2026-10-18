function [K, S, e] = dlqr(A, B, Q, R)
%DLQR Discrete-time linear-quadratic regulator.
%   [K,S,E] = DLQR(A,B,Q,R) solves the discrete Riccati equation by
%   value iteration and returns the optimal gain K, the stabilizing solution
%   S and the closed-loop eigenvalues E.
%
%   Reference: Anderson & Moore, Optimal Control: Linear Quadratic Methods,
%   1990, ch. 4.
    arguments
        A
        B
        Q
        R
    end
    S = Q;
    for it = 1:500
        Snew = Q + A' * S * A - A' * S * B * ((R + B' * S * B) \ (B' * S * A));
        d = max(max(abs(Snew - S)));
        S = Snew;
        if d < 1e-12
            break
        end
    end
    K = (R + B' * S * B) \ (B' * S * A);
    e = eig(A - B * K);
end
