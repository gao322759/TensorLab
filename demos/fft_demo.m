%% fft_demo.m — FFT 频谱分析
fs = 1000;                    % 采样率
t = 0 : 1/fs : 1 - 1/fs;       % 1 秒
signal = 0.7 * sin(2 * pi * 50 * t) + sin(2 * pi * 120 * t);
noisy = signal + 0.2 * randn(size(t));

Y = fft(noisy);
P = abs(Y / length(noisy));
P = P(1 : floor(length(P) / 2) + 1);
P(2:end-1) = 2 * P(2:end-1);
f = fs * (0 : (length(P) - 1)) / length(noisy);

[peak_val, peak_idx] = max(P(2:end));
fprintf('peak at %.1f Hz (power %.3f)\n', f(peak_idx + 1), peak_val);

figure;
subplot(2, 1, 1);
plot(t(1:100), noisy(1:100));
title('Noisy signal (first 100 samples)');

subplot(2, 1, 2);
plot(f, P);
title('Single-sided amplitude spectrum');
xlabel('Frequency (Hz)');
ylabel('|P1(f)|');
grid on;
