%% matrix_ops.m — 矩阵运算 / inv / det / eig
% 注意：magic(4) 行列式为 0（奇异），故用 magic(5)
A = magic(5);
disp('A =');
disp(A);

fprintf('sum(A) = ');
disp(sum(A));
fprintf('A'' =\n');
disp(A');

B = A * A';        % 矩阵乘
C = A .* 2;         % 逐元素
fprintf('max(B) = ');
disp(max(B(:)));

Ai = inv(A);
fprintf('det(A) = %.4f\n', det(A));
fprintf('A * Ai ≈ I: max err = %.2e\n', max(max(abs(A * Ai - eye(5)))));

[V, D] = eig(B);    % B 对称
fprintf('eigenvalues of A*A'': ');
disp(D(1, 1));
fprintf('largest eig = %.4f\n', max(diag(D)));

x = A \ ones(5, 1);  % 求解 A x = b
fprintf('residual ||Ax - b|| = %.2e\n', norm(A * x - ones(5, 1)));
