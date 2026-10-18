function K = acker(A, B, p)
%ACKER Pole placement by Ackermann's formula (single-input systems).
%   K = ACKER(A,B,P) returns the state-feedback gain placing the closed-loop
%   eigenvalues of A-B*K at the poles P.
%   Reference: Kailath, Linear Systems, 1980, ch. 6.
    arguments
        A
        B
        p
    end
    n = size(A, 1);
    if size(B, 2) ~= 1
        error('acker:multiInput', 'acker requires a single-input system (B must have one column)');
    end
    if numel(p) ~= n
        error('acker:poleCount', 'the number of poles must equal the number of states');
    end
    % desired characteristic polynomial (highest power first)
    phi = 1;
    for k = 1:numel(p)
        phi = conv(phi, [1, -p(k)]);
    end
    % controllability matrix
    Ctrb = zeros(n, n);
    col = B;
    for k = 1:n
        Ctrb(:, k) = col;
        col = A * col;
    end
    % phi(A) by Horner (phi is ordered highest power first)
    phiA = phi(1) * eye(n);
    for k = 2:n + 1
        phiA = phiA * A + phi(k) * eye(n);
    end
    e = zeros(1, n);
    e(n) = 1;
    K = e * (Ctrb \ phiA);
end
