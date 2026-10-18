%% hello_world.m — 输出与字符串基础
disp('Hello from TensorLab!');

name = "TensorLab";
version_num = 0.1;
fprintf('Running %s v%.1f\n', name, version_num);

x = 2 + 3;
y = x * 4;
fprintf('x = %d, y = %d\n', x, y);

% 字符串拼接：strcat（char 拼接，全版本兼容）
s = strcat('Tensor', 'Lab');
fprintf('concat: %s\n', s);
