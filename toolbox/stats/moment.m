function m = moment(x, order)
%MOMENT Central moment of a sample.
%   M = MOMENT(X,ORDER) returns the ORDER-th central moment of X.
    arguments
        x
        order double
    end
    x = reshape(x, numel(x), 1);
    m = mean((x - mean(x)).^order);
end
