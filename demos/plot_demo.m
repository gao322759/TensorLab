%% plot_demo.m — 绘图命令全链路（plot / subplot / hold / legend / grid）
x = linspace(0, 4 * pi, 200);
y1 = sin(x);
y2 = cos(x) .* exp(-x / 6);

figure;
subplot(2, 1, 1);
plot(x, y1, 'b-', x, y2, 'r--');
legend('sin(x)', 'cos(x)e^{-x/6}');
title('Damped oscillation');
grid on;

subplot(2, 1, 2);
stem(x(1:10:end), y1(1:10:end));
title('Stem samples');
xlabel('x');
ylabel('sin(x)');

figure;
bar([3 5 2 7 4]);
title('Bar chart');

fprintf('figures created: %d\n', length(findobj('Type', 'figure')));
