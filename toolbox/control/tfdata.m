function [num, den] = tfdata(sys)
%TFDATA Numerator and denominator of a transfer-function model.
%   [NUM,DEN] = TFDATA(SYS) returns the coefficient vectors of the SISO
%   transfer function SYS (ascending powers of s, as MATLAB).
    arguments
        sys
    end
    num = sys.Numerator;
    den = sys.Denominator;
end
