function B = ordfilt2(A, order, domain)
%ORDFILT2 2-D order-statistic filtering.
%   B = ORDFILT2(A,ORDER,DOMAIN) replaces each pixel with the ORDER-th
%   smallest value in its neighbourhood, where DOMAIN is a numeric matrix
%   whose non-zero entries select the neighbourhood (e.g. ones(3)).
%   Out-of-range neighbours are zero padded (MATLAB's default).
    arguments
        A
        order double
        domain
    end
    sz = size(A);
    ss = size(domain);
    hm = floor(ss(1) / 2);
    hn = floor(ss(2) / 2);
    cnt_dom = 0;
    for a = 1:ss(1)
        for b = 1:ss(2)
            if domain(a, b) ~= 0
                cnt_dom = cnt_dom + 1;
            end
        end
    end
    if order < 1 || order > cnt_dom
        error('ordfilt2:orderOutOfRange', 'ORDER must be in the range 1..nnz(DOMAIN)');
    end
    B = zeros(sz(1), sz(2));
    for i = 1:sz(1)
        for j = 1:sz(2)
            vals = zeros(cnt_dom, 1);
            cnt = 0;
            for a = 1:ss(1)
                for b = 1:ss(2)
                    if domain(a, b) ~= 0
                        ii = i + a - 1 - hm;
                        jj = j + b - 1 - hn;
                        cnt = cnt + 1;
                        if ii >= 1 && ii <= sz(1) && jj >= 1 && jj <= sz(2)
                            vals(cnt) = A(ii, jj);
                        end
                    end
                end
            end
            vals = sort(vals(1:cnt));
            B(i, j) = vals(order);
        end
    end
end
