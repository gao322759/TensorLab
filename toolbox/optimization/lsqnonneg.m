function x = lsqnonneg(C, d)
%LSQNONNEG Non-negative least squares.
%   X = LSQNONNEG(C,D) solves min ||C*X - D|| subject to X >= 0.
%   Active-set algorithm after Lawson & Hanson, Solving Least Squares
%   Problems, 1974, ch. 23.
    arguments
        C
        d
    end
    m = size(C, 1);
    n = size(C, 2);
    dv = reshape(d, m, 1);
    x = zeros(n, 1);
    P = zeros(n, 1);
    w = C' * dv;
    tol = 1e-12;
    maxit = 30 * (n + 1);
    for iter = 1:maxit
        jbest = 0;
        best = tol;
        for j = 1:n
            if P(j) == 0
                if w(j) > best
                    best = w(j);
                    jbest = j;
                end
            end
        end
        if jbest == 0
            break
        end
        P(jbest) = 1;

        s = zeros(n, 1);
        for inner = 1:maxit
            cnt = 0;
            sub = zeros(n, 1);
            for j = 1:n
                if P(j) == 1
                    cnt = cnt + 1;
                    sub(cnt) = j;
                end
            end
            if cnt == 0
                s = zeros(n, 1);
                break
            end
            sub = sub(1:cnt);
            Cp = C(:, sub);
            sp = Cp \ dv;
            s = zeros(n, 1);
            for k = 1:cnt
                s(sub(k)) = sp(k);
            end
            anyneg = 0;
            for k = 1:cnt
                if s(sub(k)) <= 0
                    anyneg = 1;
                end
            end
            if anyneg == 0
                break
            end
            alpha = 1e30;
            for k = 1:cnt
                if s(sub(k)) <= 0
                    den = x(sub(k)) - s(sub(k));
                    if den > 0
                        a = x(sub(k)) / den;
                        if a < alpha
                            alpha = a;
                        end
                    end
                end
            end
            for j = 1:n
                x(j) = x(j) + alpha * (s(j) - x(j));
            end
            for k = 1:cnt
                if x(sub(k)) <= tol
                    P(sub(k)) = 0;
                end
            end
        end

        x = s;
        w = C' * (dv - C * x);
    end
end
