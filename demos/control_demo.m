%% Control System toolbox demo (v0.2)
G = tf([1], [1 2 1]);
[num, den] = tfdata(G);
fprintf('tfdata den = %.1f %.1f %.1f\n', den(1), den(2), den(3));

[mag, phase, w] = bode(G);
fprintf('bode points = %d\n', numel(w));

[Gm, Pm, Wcg, Wcp] = margin(G);
fprintf('margin Pm = %.2f deg\n', Pm);

Gc = c2d(G, 0.1, 'tustin');
fprintf('c2d den = %.3f %.3f %.3f\n', Gc.Denominator(1), Gc.Denominator(2), Gc.Denominator(3));

Gz = c2d(G, 0.1, 'zoh');
fprintf('c2d zoh order = %d\n', numel(Gz.Denominator));

T = tf([1 1], [1 3 2]);
Tm = minreal(T);
fprintf('minreal order = %d\n', numel(Tm.Denominator) - 1);

A = [0 1; -2 -3];
B = [0; 1];
C = [1 0];
D = 0;
S = ss(A, B, C, D);
[Ao, Bo, Co, Do] = ssdata(S);
fprintf('ssdata A11 = %.1f D = %.1f\n', Ao(1,1), Do);

[z, p, k] = ss2zp(A, B, C, D);
fprintf('ss2zp zeros = %d gain = %.3f\n', numel(z), k);

[Ad, Bd, Cd, Dd] = zp2ss([], [-1], 2);
fprintf('zp2ss A = %.2f B = %.2f C = %.2f\n', Ad(1,1), Bd(1,1), Cd(1,1));

[K, Sric, e] = dlqr([1 1; 0 1], [0.5; 1], eye(2), 1);
fprintf('dlqr K = %.3f %.3f\n', K(1,1), K(1,2));

[L, P] = kalman(S, eye(2), 1);
fprintf('kalman L = %.4f P = %.4f\n', L(1,1), P(1,1));

[md, ph, wn] = nichols(G);
fprintf('nichols points = %d\n', numel(wn));

[y, t] = step(G);
fprintf('step final = %.3f\n', y(numel(y)));
fprintf('control toolbox demo OK\n');
