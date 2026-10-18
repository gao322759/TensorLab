function D = bwdist(BW)
%BWDIST Euclidean distance transform of a binary image.
%   D = BWDIST(BW) returns, for every pixel, the Euclidean distance to the
%   nearest non-zero (foreground) pixel.  If BW has no foreground pixels,
%   D is Inf everywhere.
%
%   Implemented by an exact brute-force search over foreground pixels
%   (O(W*H*F)); adequate for moderate image sizes.
    arguments
        BW
    end
    fg = BW ~= 0;
    sz = size(BW);
    D = zeros(sz(1), sz(2));
    fi = zeros(numel(BW), 1);
    fj = zeros(numel(BW), 1);
    nf = 0;
    for i = 1:sz(1)
        for j = 1:sz(2)
            if fg(i, j)
                nf = nf + 1;
                fi(nf) = i;
                fj(nf) = j;
            end
        end
    end
    if nf == 0
        for i = 1:sz(1)
            for j = 1:sz(2)
                D(i, j) = Inf;
            end
        end
        return
    end
    big = 1e30;
    for i = 1:sz(1)
        for j = 1:sz(2)
            best = big;
            for k = 1:nf
                di = i - fi(k);
                dj = j - fj(k);
                d2 = di * di + dj * dj;
                if d2 < best
                    best = d2;
                end
            end
            D(i, j) = sqrt(best);
        end
    end
end
