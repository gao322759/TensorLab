function B = ordfilt2(A, order, domain)
%ORDFILT2 2-D order-statistic filtering.
%   B = ORDFILT2(A,ORDER,DOMAIN) replaces each pixel with the ORDER-th
%   smallest value in its neighbourhood, where DOMAIN is a numeric matrix
%   whose non-zero entries select the neighbourhood (e.g. ones(3)).
%   Out-of-range neighbours are clamped (MATLAB's 'replicate' default).
    arguments
        A
        order double
        domain
    end
    sz = size(A);
    ss = size(domain);
    hm = floor(ss(1) / 2);
    hn = floor(ss(2) / 2);
    B = zeros(sz(1), sz(2));
    for i = 1:sz(1)
        for j = 1:sz(2)
            vals = zeros(ss(1) * ss(2), 1);
            cnt = 0;
            for a = 1:ss(1)
                for b = 1:ss(2)
                    if domain(a, b) ~= 0
                        ii = i + a - 1 - hm;
                        jj = j + b - 1 - hn;
                        if ii < 1
                            ii = 1;
                        end
                        if ii > sz(1)
                            ii = sz(1);
                        end
                        if jj < 1
                            jj = 1;
                        end
                        if jj > sz(2)
                            jj = sz(2);
                        end
                        cnt = cnt + 1;
                        vals(cnt) = A(ii, jj);
                    end
                end
            end
            vals = sort(vals(1:cnt));
            B(i, j) = vals(order);
        end
    end
end
