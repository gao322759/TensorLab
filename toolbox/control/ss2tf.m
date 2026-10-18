function [num, den] = ss2tf(A, B, C, D, iu)
%SS2TF State-space to transfer function (SISO).
%   [NUM,DEN] = SS2TF(A,B,C,D) returns the numerator and denominator of
%   C*(sI-A)^-1*B + D, normalized so DEN(1) = 1.
%
%   Uses the Faddeev-LeVerrier algorithm for det(sI-A) and adj(sI-A)
%   (Gantmacher, The Theory of Matrices, vol. 1, ch. 4).
%   Subset: single-input single-output systems only (IU is accepted for
%   signature compatibility and ignored).
%   Note: avoids RESHAPE (kernel stack-overflow pitfall, see docs/kernel-notes.md).
    arguments
        A
        B
        C
        D
        iu double = 1
    end
    n = size(A, 1);
    if size(B, 2) ~= 1
        error('ss2tf:multiInput', 'ss2tf currently supports single-input systems only');
    end
    if size(C, 1) ~= 1
        error('ss2tf:multiOutput', 'ss2tf currently supports single-output systems only');
    end

    % Trivial first-order case (avoids 1x1 variable-index assignment, see
    % docs/kernel-notes.md).
    if n == 1
        d0 = D;
        if numel(D) > 1
            d0 = D(1, 1);
        end
        a = A;
        if numel(A) > 1
            a = A(1, 1);
        end
        b1 = B;
        if numel(B) > 1
            b1 = B(1, 1);
        end
        c1 = C;
        if numel(C) > 1
            c1 = C(1, 1);
        end
        den = [1, -a];
        num = [d0, c1 * b1 - d0 * a];
        return
    end

    % Store the M_k matrices column-by-column in Ms (no cell arrays).
    Ms = zeros(n * n, n);
    for i = 1:n
        Ms((i - 1) * n + i, 1) = 1;
    end

    c = zeros(1, n + 1);
    c(1) = 1;
    for k = 1:n
        M = zeros(n, n);
        for i = 1:n
            for j = 1:n
                M(i, j) = Ms((j - 1) * n + i, k);
            end
        end
        AM = A * M;
        ck = -trace(AM) / k;
        c(k + 1) = ck;
        if k < n
            Mc = AM + ck * eye(n);
            for i = 1:n
                for j = 1:n
                    Ms((j - 1) * n + i, k + 1) = Mc(i, j);
                end
            end
        end
    end

    den = c;
    d0 = D;
    if numel(D) > 1
        d0 = D(1, 1);
    end
    num = zeros(1, n + 1);
    num(1) = d0;
    for j = 1:n
        M = zeros(n, n);
        for i = 1:n
            for l = 1:n
                M(i, l) = Ms((l - 1) * n + i, j);
            end
        end
        num(j + 1) = C * M * B + d0 * c(j + 1);
    end
end
