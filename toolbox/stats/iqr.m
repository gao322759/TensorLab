function r = iqr(x)
%IQR Interquartile range.
%   R = IQR(X) returns the difference between the 75th and 25th percentiles.
    arguments
        x
    end
    q = prctile(x, [25 75]);
    r = q(2) - q(1);
end
