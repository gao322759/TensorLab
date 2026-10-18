function [A, B, C, D] = ssdata(sys)
%SSDATA State-space matrices of an SS model.
%   [A,B,C,D] = SSDATA(SYS) returns the four matrices of the state-space
%   model SYS.
    arguments
        sys
    end
    A = sys.A;
    B = sys.B;
    C = sys.C;
    D = sys.D;
end
