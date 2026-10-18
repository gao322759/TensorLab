function B = imrotate(A, angle, method)
%IMROTATE Rotate an image counterclockwise by ANGLE degrees.
%   B = IMROTATE(A,ANGLE) rotates with nearest-neighbour sampling.
%   B = IMROTATE(A,ANGLE,METHOD) with METHOD 'nearest' or 'bilinear'.
%   The output canvas is enlarged to fit the rotated image; outside pixels
%   are zero.  Single-plane (grayscale) images only.
    arguments
        A
        angle double
        method = 'nearest'
    end
    sz = size(A);
    m = sz(1);
    n = sz(2);
    th = angle * pi / 180;
    ct = cos(th);
    st = sin(th);
    mout = ceil(abs(m * ct) + abs(n * st));
    nout = ceil(abs(m * st) + abs(n * ct));
    B = zeros(mout, nout);
    ci = (m + 1) / 2;
    cj = (n + 1) / 2;
    coi = (mout + 1) / 2;
    coj = (nout + 1) / 2;
    linear = strcmpi(method, 'bilinear') || strcmpi(method, 'linear');
    for i = 1:mout
        for j = 1:nout
            xi = ct * (i - coi) + st * (j - coj) + ci;
            xj = -st * (i - coi) + ct * (j - coj) + cj;
            if linear
                i0 = floor(xi);
                j0 = floor(xj);
                di = xi - i0;
                dj = xj - j0;
                i1 = i0 + 1;
                j1 = j0 + 1;
                if i0 >= 1 && j0 >= 1 && i1 <= m && j1 <= n
                    B(i, j) = A(i0, j0) * (1 - di) * (1 - dj) ...
                            + A(i0, j1) * (1 - di) * dj ...
                            + A(i1, j0) * di * (1 - dj) ...
                            + A(i1, j1) * di * dj;
                end
            else
                ii = round(xi);
                jj = round(xj);
                if ii >= 1 && ii <= m && jj >= 1 && jj <= n
                    B(i, j) = A(ii, jj);
                end
            end
        end
    end
end
