function [A, B, C, D] = zp2ss(z, p, k)
%ZP2SS Zero-pole-gain to state-space.
%   [A,B,C,D] = ZP2SS(Z,P,K) converts a SISO zero-pole-gain model to
%   controllable canonical state-space form (via ZP2TF and TF2SS).
    arguments
        z
        p
        k double = 1
    end
    [num, den] = zp2tf(z, p, k);
    [A, B, C, D] = tf2ss(num, den);
end
