%% Optimization toolbox demo (v0.2)
o = optimset('TolX', 1e-8, 'MaxIter', 200);
fprintf('optimget TolX = %.3g\n', optimget(o, 'TolX'));

[x1, f1] = fminsearch(@(v) (v(1) - 1)^2 + (v(2) + 2)^2, [0; 0]);
fprintf('fminsearch x = %.3f %.3f f = %.4f\n', x1(1), x1(2), f1);

[x2, f2, flag2] = fmincon(@(v) v(1)^2 + v(2)^2, [2; 2], [], [], [1 1], 1, [0; 0], []);
fprintf('fmincon x = %.3f %.3f flag = %d\n', x2(1), x2(2), flag2);

C = [1 0; 0 1; 1 1];
d = [1; 2; 3];
[x3, rn3] = lsqlin(C, d, [1 1], 2);
fprintf('lsqlin x = %.3f %.3f resnorm = %.3f\n', x3(1), x3(2), rn3);

[x4, f4] = intlinprog([-3; -2], [1 2], [1 1; 1 3], [4; 6], [], [], [0; 0], []);
fprintf('intlinprog x = %.0f %.0f f = %.1f\n', x4(1), x4(2), f4);

x5 = lsqnonneg([1 0; 0 1; 1 1], [-1; 2; 1]);
fprintf('lsqnonneg x = %.3f %.3f\n', x5(1), x5(2));

rng(5);
[x6, f6] = ga(@(v) (v(1) - 1)^2 + (v(2) - 2)^2, 2, [], [], [], [], [0; 0], [3; 3]);
fprintf('ga x = %.2f %.2f\n', x6(1), x6(2));

rng(6);
[x7, f7] = simulannealbnd(@(v) (v(1) - 1)^2 + (v(2) - 2)^2, [0.5; 0.5], [0; 0], [3; 3]);
fprintf('simulannealbnd x = %.2f %.2f\n', x7(1), x7(2));

rng(7);
[x8, f8] = particleswarm(@(v) (v(1) - 1)^2 + (v(2) - 2)^2, 2, [0; 0], [3; 3]);
fprintf('particleswarm x = %.2f %.2f\n', x8(1), x8(2));

[x9, f9] = patternsearch(@(v) (v(1) - 1)^2 + (v(2) - 2)^2, [0.5; 0.5], [], [], [], [], [0; 0], [3; 3]);
fprintf('patternsearch x = %.3f %.3f\n', x9(1), x9(2));
fprintf('optimization toolbox demo OK\n');
