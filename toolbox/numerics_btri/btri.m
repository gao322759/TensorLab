function x = btri(A, b, nb)
%BTRI Block-tridiagonal direct solve (block Thomas algorithm).
%
%   X = btri(A, B, NB) solves A*X = B where the N x N matrix A is block
%   tridiagonal with NB diagonal blocks of size q = N/NB: the 5-point FVM
%   stencil on an (q x NB) cell grid with the column-major unknown
%   ordering block i = rows (i-1)q+1 : i*q (sicpvt solve_species layout).
%   Direct method — no convergence parameter; arithmetic differs from the
%   dense backslash only in operation order.
%
%   Clean-room textbook algorithm (block tridiagonal elimination, e.g.
%   Golub & Van Loan, Matrix Computations §4.5 — Thomas generalization).
%
%   Implementation notes (RunMat 0.6.2 constrained, canaries K20-K27):
%   block inverses and modified right-hand sides are accumulated as
%   row-block stacks with vertical concat; block reads are row-range;
%   every right-hand-side block is reshaped to a column before matmul
%   (1-D range reads normalise to row shape, K27).

[m, n] = size(A);
if m ~= n
    error('tl:numerics', 'btri requires a square matrix');
end
N = m;
if mod(N, nb) ~= 0
    error('tl:numerics', 'btri: block count must divide N');
end
q = N / nb;
bc = reshape(b, N, 1);

% ---- forward elimination ----------------------------------------------------
r1 = 1;
r2 = q;
Ci = A(r1:r2, r1:r2);
Ti = inv(Ci);
IS = Ti;                       % row-block stack: block i = inv(S_i), q x q
fi = bc(r1:r2);
fic = reshape(fi, q, 1);
gi = Ti * fic;
G = reshape(gi, 1, q);         % row-block stack (reshape materialises; K26)
for i = 2:nb
    rp1 = (i - 2) * q + 1;
    rp2 = (i - 1) * q;
    r1 = (i - 1) * q + 1;
    r2 = i * q;
    Ci = A(r1:r2, r1:r2);
    Ep = A(r1:r2, rp1:rp2);    % lower block E_i (couples to block i-1)
    Fp = A(rp1:rp2, r1:r2);    % upper block F_{i-1}
    Tp = IS(rp1:rp2, :);       % inv(S_{i-1})
    S = Ci - Ep * (Tp * Fp);
    Ti = inv(S);
    IS = [IS; Ti];
    gpc = reshape(G(i - 1, :), q, 1);
    fic = reshape(bc(r1:r2), q, 1);
    gi = Ti * (fic - Ep * gpc);
    G = [G; gi'];
end

% ---- back substitution ------------------------------------------------------
x = zeros(1, N);
x(N - q + 1:N) = G(nb, :);
for i = nb-1:-1:1
    r1 = (i - 1) * q + 1;
    r2 = i * q;
    rn1 = i * q + 1;
    rn2 = (i + 1) * q;
    Fp = A(r1:r2, rn1:rn2);    % upper block F_i
    Tp = IS(r1:r2, :);         % inv(S_i)
    xn = x(rn1:rn2);           % row (K27)
    xnc = reshape(xn, q, 1);
    gic = reshape(G(i, :), q, 1);
    xi = gic - Tp * (Fp * xnc);
    x(r1:r2) = reshape(xi, 1, q);
end
end
