function [L, P] = kalman(sys, Q, R)
%KALMAN Continuous-time steady-state Kalman estimator gain.
%   [L,P] = KALMAN(SYS,Q,R) solves the continuous estimator Riccati
%   equation for the state-space model SYS (A,B,C) and returns the gain L
%   and the error covariance P. MATLAB returns an estimator model as the
%   first output; this kernel build returns the numeric gain.
%
%   Hamiltonian invariant-subspace method
%   (Anderson & Moore, Optimal Filtering, 1979, ch. 4).
    arguments
        sys
        Q
        R
    end
    A = sys.A;
    B = sys.B;
    C = sys.C;
    n = size(A, 1);
    Ri = 1 / R;
    if n == 1
        a = A;
        if numel(A) > 1
            a = A(1, 1);
        end
        c = C;
        if numel(C) > 1
            c = C(1, 1);
        end
        q = Q;
        if numel(Q) > 1
            q = Q(1, 1);
        end
        coef = c * c * Ri;
        P = (a + sqrt(a * a + coef * q)) / coef;
        L = P * c / (R + c * P * c);
        return
    end
    H = zeros(2 * n, 2 * n);
    for i = 1:n
        for j = 1:n
            H(i, j) = A(j, i);
            H(n + i, n + j) = -A(i, j);
        end
    end
    for i = 1:n
        for j = 1:n
            s = 0;
            for k = 1:size(C, 1)
                s = s + C(k, i) * Ri * C(k, j);
            end
            H(i, n + j) = -s;
        end
    end
    for i = 1:n
        for j = 1:n
            H(n + i, j) = -Q(i, j);
        end
    end

    [V, D] = eig(H);
    V1 = zeros(n, n);
    V2 = zeros(n, n);
    col = 0;
    for k = 1:2 * n
        if real(D(k, k)) < 0
            col = col + 1;
            for i = 1:n
                V1(i, col) = real(V(i, k));
                V2(i, col) = real(V(n + i, k));
            end
        end
    end
    if col ~= n
        error('kalman:riccati', 'could not isolate the stable invariant subspace');
    end
    P = V2 / V1;
    P = (P + P') / 2;
    L = P * C' / (R + C * P * C');
end
