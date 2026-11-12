function RGB = label2rgb(L, map)
%LABEL2RGB Convert a label matrix to a colour image.
%   RGB = LABEL2RGB(L) maps labels 1..max(L) with the jet colormap and uses
%   white for the background label 0, matching MATLAB.  The result is uint8.
%   RGB = LABEL2RGB(L,MAP) uses the N-by-3 colour map MAP; MAP may also be a
%   colormap name such as 'jet'.
    arguments
        L
        map = []
    end
    if isempty(L)
        error('label2rgb:emptyInput', 'L must be non-empty');
    end
    sz = size(L);
    nmax = round(max(max(L)));
    if nmax < 1
        nmax = 1;
    end
    if isempty(map)
        map = tl_jet(nmax);
    elseif strcmp(class(map), 'char')
        map = tl_jet(nmax);
    end
    nm = size(map, 1);
    RGB = zeros(sz(1), sz(2), 3);
    for i = 1:sz(1)
        for j = 1:sz(2)
            k = round(L(i, j));
            if k <= 0
                RGB(i, j, 1) = 1;
                RGB(i, j, 2) = 1;
                RGB(i, j, 3) = 1;
            else
                if k > nm
                    k = nm;
                end
                c = map(k, :);
                RGB(i, j, 1) = c(1);
                RGB(i, j, 2) = c(2);
                RGB(i, j, 3) = c(3);
            end
        end
    end
    RGB = uint8(round(255 * RGB));
end

function J = tl_jet(m)
%TL_JET MATLAB-compatible jet colormap.
    n = ceil(m / 4);
    nu = 3 * n - 1;
    u = zeros(nu, 1);
    for i = 1:n
        u(i) = i / n;
    end
    for i = 1:(n - 1)
        u(n + i) = 1;
    end
    for i = 1:n
        u(2 * n - 1 + i) = (n - i + 1) / n;
    end
    g0 = ceil(n / 2) - (mod(m, 4) == 1);
    J = zeros(m, 3);
    cnt = 0;
    for i = 1:nu
        idx = g0 + i;
        if idx <= m
            cnt = cnt + 1;
            J(idx, 2) = u(i);
        end
    end
    cnt = 0;
    for i = 1:nu
        idx = g0 + n + i;
        if idx <= m
            cnt = cnt + 1;
            J(idx, 1) = u(i);
        end
    end
    kb = 0;
    for i = 1:nu
        idx = g0 - n + i;
        if idx >= 1 && idx <= m
            kb = kb + 1;
        end
    end
    cb = 0;
    for i = 1:nu
        idx = g0 - n + i;
        if idx >= 1 && idx <= m
            cb = cb + 1;
            J(idx, 3) = u(nu - kb + cb);
        end
    end
end
