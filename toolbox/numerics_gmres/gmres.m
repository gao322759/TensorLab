function [x, flag, relres, iter] = gmres(A, b, restart, tol, maxit, x0)
%GMRES Restarted GMRES iterative solver (TensorLab numerics, plan v0.4-2.2).
%
%   [X, FLAG, RELRES, ITER] = gmres(A, B, RESTART, TOL, MAXIT, X0) attempts
%   to solve A*X = B for the square matrix A with restarted GMRES
%   (Saad, Iterative Methods for Sparse Linear Systems, ch. 6 — clean-room
%   textbook algorithm).  A must be a full (dense) matrix in this
%   release: true sparse storage is a kernel item (plan.md 2.2, blocked
%   on the host builtin-registration API) and the iterative method here
%   only ever needs matrix-vector products, so the dense assembly is
%   acceptable up to a few thousand unknowns.
%
%   Arguments (MATLAB-compatible order; omit trailing ones):
%     restart  inner iteration count per cycle   (default N)
%     tol      relative residual tolerance        (default 1e-6)
%     maxit    outer (cycle) count                (default ceil(N/restart))
%     x0       initial guess                      (default zeros)
%
%   Outputs:
%     x       solution vector (N x 1)
%     flag    0 = converged, 1 = maxit exhausted, 2 = breakdown/stagnation
%     relres  final relative residual  ||b - A x|| / ||b||
%     iter    [outer_cycles, total_inner_iterations]
%
%   Implementation notes (RunMat 0.6.2 constrained, kernel canaries
%   K20-K27): the Krylov basis is ONE long column vector with vertical
%   concat growth; 1-D block reads normalise to ROW shape (K27), so every
%   matrix-vector product is wrapped in reshape(., N, 1) and the basis
%   blocks are used as rows for the modified Gram-Schmidt inner products
%   (sum(x .* y) form).  The Hessenberg matrix is accumulated as H' in
%   row form (Hrows = [Hrows; hrow]), so the cycle least-squares
%   min || beta e1 - H y || is solved via
%       y = (Hrows*Hrows') \ (Hrows*e1)
%   — no slicing, no transposes of computed values (K26), no Givens
%   indexed stores.

arguments
    A
    b
    restart = []
    tol = 1e-6
    maxit = []
    x0 = []
end

[m, n] = size(A);
if m ~= n
    error('tl:numerics', 'gmres requires a square matrix');
end
b = reshape(b, numel(b), 1);
if numel(b) ~= n
    error('tl:numerics', 'gmres: b must have N rows');
end
if isempty(restart)
    restart = n;
end
if restart < 1
    restart = 1;
end
if restart > n
    restart = n;
end
if isempty(maxit)
    maxit = ceil(n / restart);
end
if isempty(x0)
    x = zeros(n, 1);
else
    x = reshape(x0, n, 1);
end

normb = sqrt(sum(b .* b));
flag = 1;
relres = 1;
outer_used = 0;
inner_used = 0;
if normb == 0
    x = zeros(n, 1);
    flag = 0;
    relres = 0;
    iter = [0, 0];
    return;
end

r = b - A * x;
nrm = sqrt(sum(r .* r));
if nrm / normb <= tol
    flag = 0;
    relres = nrm / normb;
    iter = [0, 0];
    return;
end

e1 = zeros(restart + 1, 1);
e1(1) = 1;

for outer = 1:maxit
    outer_used = outer;
    beta = nrm;
    % Krylov basis: one long ROW vector with horizontal concat growth
    % (vertical concat of columns breaks 1-D block reads — kernel
    % canary K27 family); 1-D block reads of rows are safe.
    v1 = r / beta;
    Vc = reshape(v1, 1, n);
    k = restart;
    brk = 0;
    Hrows = zeros(0, restart + 1);
    for j = 1:restart
        % w = A * v_j  (v_j read is a row; reshape to column for matmul)
        r1 = (j - 1) * n + 1;
        r2 = j * n;
        vj = Vc(r1:r2);
        vjc = reshape(vj, n, 1);
        w = A * vjc;
        % orthogonalise against v_1..v_j (modified Gram-Schmidt, row form)
        wr = reshape(w, 1, n);
        hrow = zeros(1, restart + 1);
        for i = 1:j
            ri = (i - 1) * n + 1;
            rj = i * n;
            vi = Vc(ri:rj);
            h = sum(wr .* vi);
            hrow(i) = h;
            wr = wr - h * vi;
        end
        hn = sqrt(sum(wr .* wr));
        hrow(j + 1) = hn;
        Hrows = [Hrows; hrow];
        inner_used = inner_used + 1;
        if hn <= 1e-14 * (1 + beta)
            % happy breakdown: the Krylov space is invariant
            brk = 1;
            k = j;
            break;
        end
        wr = wr / hn;
        Vc = [Vc, wr];
    end

    % min || beta e1 - H y ||: Hrows = H' (row form), so the normal
    % equations read  y = (Hrows Hrows') \ (Hrows (beta e1))  — square,
    % non-singular for both the full cycle and the happy-breakdown cut.
    He = Hrows * (beta * e1);
    Hh = Hrows * Hrows';
    y = Hh \ He;
    for i = 1:k
        r1 = (i - 1) * n + 1;
        r2 = i * n;
        vi = Vc(r1:r2);
        vic = reshape(vi, n, 1);
        x = x + y(i) * vic;
    end

    r = b - A * x;
    nrm = sqrt(sum(r .* r));
    relres = nrm / normb;
    if relres <= tol
        flag = 0;
        break;
    end
    if brk == 1
        % invariant subspace reached without meeting tol
        flag = 2;
        break;
    end
end

if flag == 1 && relres <= tol
    flag = 0;
end
iter = [outer_used, inner_used];
end
