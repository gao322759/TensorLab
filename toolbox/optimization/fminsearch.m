function [x, fval] = fminsearch(fun, x0)
%FMINSEARCH Nelder-Mead simplex direct search (unconstrained).
%   X = FMINSEARCH(FUN,X0) minimizes FUN starting at X0.
%   [X,FVAL] = FMINSEARCH(...) also returns the objective value.
%
%   Algorithm: Nelder & Mead, Computer Journal 7(4), 1965.
    arguments
        fun
        x0
    end
    n = numel(x0);
    if n < 2
        % Kernel limitation: scalars are not slice-assignable, so the simplex
        % construction below cannot run. Use fminbnd for 1-D problems.
        error('fminsearch:scalarX0', 'x0 must have at least 2 elements (use fminbnd for 1-D)');
    end
    x0 = x0(:);

    % Initial simplex: MATLAB perturbs each component by 5% (0.00025 if zero).
    V = zeros(n, n + 1);
    V(:, 1) = x0;
    for k = 1:n
        y = x0;
        if y(k) ~= 0
            y(k) = y(k) * 1.05;
        else
            y(k) = 0.00025;
        end
        V(:, k + 1) = y;
    end
    fv = zeros(1, n + 1);
    for k = 1:n + 1
        fv(k) = fun(V(:, k));
    end

    alpha = 1;
    gamma = 2;
    rho = 0.5;
    sigma = 0.5;
    for iter = 1:400
        [fv, idx] = sort(fv);
        V = V(:, idx);
        spread = max(max(abs(V(:, 2:end) - V(:, 1))));
        if spread < 1e-9
            break;
        end
        c = mean(V(:, 1:n), 2);
        xr = c + alpha * (c - V(:, n + 1));
        fr = fun(xr);
        if fr < fv(1)
            xe = c + gamma * (xr - c);
            fe = fun(xe);
            if fe < fr
                V(:, n + 1) = xe;
                fv(n + 1) = fe;
            else
                V(:, n + 1) = xr;
                fv(n + 1) = fr;
            end
        elseif fr < fv(n)
            V(:, n + 1) = xr;
            fv(n + 1) = fr;
        else
            xc = c + rho * (V(:, n + 1) - c);
            fc = fun(xc);
            if fc < fv(n + 1)
                V(:, n + 1) = xc;
                fv(n + 1) = fc;
            else
                for k = 2:n + 1
                    V(:, k) = V(:, 1) + sigma * (V(:, k) - V(:, 1));
                    fv(k) = fun(V(:, k));
                end
            end
        end
    end
    [fval, idx] = min(fv);
    x = V(:, idx);
end
