function K = place(A, B, p)
%PLACE State-feedback pole placement.
%   K = PLACE(A,B,P) places the closed-loop eigenvalues of A-B*K at P.
%   Subset: single-input systems only (implemented via Ackermann's formula);
%   multi-input systems are not supported.
    arguments
        A
        B
        p
    end
    if size(B, 2) ~= 1
        error('place:multiInput', 'place currently supports single-input systems only');
    end
    K = acker(A, B, p);
end
