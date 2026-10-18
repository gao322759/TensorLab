%% solve_demo.m — 线性方程组 A\b
A = [3 1 -1; 2 5 2; 1 -1 4];
b = [10; 18; 7];

x = A \ b;
fprintf('solution x =\n');
disp(x);
fprintf('check: A*x - b =\n');
disp(A * x - b);

fprintf('cond(A) = %.3f\n', cond(A));
fprintf('norm(x) = %.6f\n', norm(x));

% 超定最小二乘
Am = [1 1; 1 2; 1 3; 1 4];
bm = [1; 2; 2.1; 3.9];
xm = Am \ bm;
fprintf('least squares slope = %.4f, intercept = %.4f\n', xm(2), xm(1));
