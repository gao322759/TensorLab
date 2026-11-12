function [x, flag, relres, iter] = pcg(A, b, tol, maxit, x0)
%PCG Preconditioned-free conjugate gradient solver (TensorLab numerics).
%
%   [X, FLAG, RELRES, ITER] = pcg(A, B, TOL, MAXIT, X0) solves A*X = B for
%   the symmetric positive-definite square matrix A with the conjugate
%   gradient method (Hestenes-Stiefel / Saad ch. 9 — clean-room textbook
%   algorithm).  A must be a full (dense) matrix in this release: true
%   sparse storage is a kernel item (plan.md 2.2).
%
%   Arguments (MATLAB-compatible order):
%     tol    relative residual tolerance (default 1e-6)
%     maxit  iteration cap                (default N)
%     x0     initial guess                (default zeros)
%
%   Outputs:
%     x       solution vector
%     flag    0 = converged, 1 = maxit exhausted, 2 = stagnation
%     relres  final relative residual ||b - A x|| / ||b||
%     iter    iterations taken
%
%   Implementation uses only whole-vector operations and scalar (non-
%   indexed) stores, the proven-safe RunMat 0.6.2 subset.

arguments
    A
    b
    tol = 1e-6
    maxit = []
    x0 = []
end

[m, n] = size(A);
if m ~= n
    error('tl:numerics', 'pcg requires a square matrix');
end
b = reshape(b, numel(b), 1);
if numel(b) ~= n
    error('tl:numerics', 'pcg: b must have N rows');
end
if isempty(maxit)
    maxit = n;
end
if isempty(x0)
    x = zeros(n, 1);
else
    x = reshape(x0, n, 1);
end

normb = sqrt(b' * b);
flag = 1;
relres = 1;
used = 0;
if normb == 0
    x = zeros(n, 1);
    flag = 0;
    relres = 0;
    iter = 0;
    return;
end

r = b - A * x;
nrm = sqrt(r' * r);
if nrm / normb <= tol
    flag = 0;
    relres = nrm / normb;
    iter = 0;
    return;
end

p = r;
rho = nrm * nrm;
for k = 1:maxit
    used = k;
    ap = A * p;
    pap = p' * ap;
    if pap <= 0
        flag = 2;
        break;
    end
    alpha = rho / pap;
    x = x + alpha * p;
    r = r - alpha * ap;
    nrm = sqrt(r' * r);
    relres = nrm / normb;
    if relres <= tol
        flag = 0;
        break;
    end
    rho_new = nrm * nrm;
    beta = rho_new / rho;
    rho = rho_new;
    p = r + beta * p;
    if abs(alpha) * abs(beta) == 0 && nrm == nrm
        % stagnation guard: rho collapsed without convergence
        if rho <= 0
            flag = 2;
            break;
        end
    end
end

if flag == 1 && relres <= tol
    flag = 0;
end
iter = used;
end
