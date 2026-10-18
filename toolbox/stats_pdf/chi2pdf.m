function y = chi2pdf(x, nu)
%CHI2PDF Chi-square probability density function.
%   Y = CHI2PDF(X,NU) returns the density of the chi-square distribution
%   with NU degrees of freedom.
    arguments
        x
        nu
    end
    if numel(x) == 1
        v = x(1);
        if v < 0
            y = 0;
        elseif v == 0
            if nu < 2
                y = Inf;
            elseif nu == 2
                y = 0.5;
            else
                y = 0;
            end
        else
            y = exp((nu / 2 - 1) * log(v) - v / 2 - (nu / 2) * log(2) - gammaln(nu / 2));
        end
    else
        y = zeros(1, numel(x));
        for i = 1:numel(x)
            v = x(i);
            val = 0;
            if v > 0
                val = exp((nu / 2 - 1) * log(v) - v / 2 - (nu / 2) * log(2) - gammaln(nu / 2));
            end
            y(i) = val;
        end
    end
end
