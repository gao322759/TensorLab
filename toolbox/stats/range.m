function r = range(x)
%RANGE Difference between the maximum and minimum values.
%   R = RANGE(X) returns max(X) - min(X).
    arguments
        x
    end
    r = max(x) - min(x);
end
