function h = freqs(b, a, w)
%FREQS Analog filter frequency response.
%   H = FREQS(B,A,W) evaluates H(s) = B(s)/A(s) at s = j*W.
    arguments
        b
        a
        w
    end
    s = 1i * w;
    h = polyval(b, s) ./ polyval(a, s);
end
