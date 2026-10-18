%% Signal Processing toolbox demo (v0.2)
x = [1 2 3 4 3 2 1];
r = xcorr(x);
fprintf('xcorr peak = %.1f\n', r(7));

d = dct(x);
xr = idct(d);
err1 = 0;
for k = 1:numel(x)
    e = abs(xr(k) - x(k));
    if e > err1
        err1 = e;
    end
end
fprintf('dct/idct roundtrip err = %.3g\n', err1);

A = [1 2; 3 4];
B = [1 0; 0 1];
C = convn(A, B);
fprintf('convn 3x3 sum = %.1f\n', sum(sum(C)));

h = freqs([1], [1 1], 1);
fprintf('freqs |H(1j)| = %.4f\n', abs(h));

y = upfirdn([1 2 3], [1 1], 1, 1);
fprintf('upfirdn = %.1f %.1f %.1f %.1f\n', y(1), y(2), y(3), y(4));

N = 32;
sig = zeros(1, N);
for k = 1:N
    sig(k) = cos(2 * pi * 5 * (k - 1) / N);
end
[pxx, f] = pspectrum(sig, 32);
pk = 0;
pkf = 0;
for k = 1:numel(pxx)
    if pxx(k) > pk
        pk = pxx(k);
        pkf = f(k);
    end
end
fprintf('pspectrum peak f = %.1f p = %.4f\n', pkf, pk);

z = czt(sig, 4, exp(-2i * pi / N), 1);
fprintf('czt(4) |y1| = %.4f\n', abs(z(1)));

g = goertzel(sig, 6);
fprintf('goertzel bin 6 |Y| = %.4f\n', abs(g));

[nk, Wn, beta, ftype] = kaiserord([0 0.2 0.3 1], [1 0], [0.01 0.001], 2);
fprintf('kaiserord n = %d beta = %.3f\n', nk, beta);

b = fir2(10, [0 0.3 0.4 1], [1 1 0 0]);
fprintf('fir2 dc gain = %.3f\n', sum(b));

[S, F, T] = stft(sig, 32, 8, 4, 8);
sig2 = istft(S, 32, 8, 4, 8);
err2 = 0;
for k = 2:numel(sig) - 1
    e = abs(sig2(k) - sig(k));
    if e > err2
        err2 = e;
    end
end
fprintf('stft/istft reconstruction err = %.3g\n', err2);
fprintf('signal toolbox demo OK\n');
