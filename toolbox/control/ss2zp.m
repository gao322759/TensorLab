function [z, p, k] = ss2zp(A, B, C, D)
%SS2ZP State-space to zero-pole-gain.
%   [Z,P,K] = SS2ZP(A,B,C,D) converts a SISO state-space model to zeros,
%   poles and gain (via SS2TF followed by TF2ZP).
    arguments
        A
        B
        C
        D
    end
    [num, den] = ss2tf(A, B, C, D);
    [z, p, k] = tf2zp(num, den);
end
