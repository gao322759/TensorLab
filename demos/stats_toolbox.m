%% Statistics toolbox demo (v0.2)
y = [1; 2; 3; 4; 5; 6];
g = [1; 1; 1; 2; 2; 2];
[p, F] = anova1(y, g);
fprintf('anova1 p = %.5f F = %.3f\n', p, F);

[p2, H] = kruskalwallis(y, g);
fprintf('kruskalwallis p = %.5f H = %.3f\n', p2, H);

fprintf('chi2pdf(2,2) = %.5f\n', chi2pdf(2, 2));
fprintf('fcdf(1,1,1) = %.4f\n', fcdf(1, 1, 1));

ph = mle([2 4 4 4 5 5 7 9]);
fprintf('mle mu = %.2f sigma = %.2f\n', ph(1), ph(2));

X = [1; 2; 3; 4; 5];
yv = [2.1; 4.0; 6.2; 7.9; 10.1];
[b, dev] = glmfit(X, yv, 'normal');
fprintf('glmfit b1 = %.3f dev = %.4f\n', b(2), dev);

model = @(beta, x) beta(1) * exp(beta(2) * x);
xe = [0; 1; 2; 3; 4];
ye = [2.0; 3.3; 5.4; 8.9; 14.8];
bh = nlinfit(xe, ye, model, [1; 0.5]);
fprintf('nlinfit a = %.3f b = %.3f\n', bh(1), bh(2));

Xm = [1 2; 2 4; 3 5; 4 7; 5 8];
[coeff, score, latent] = pca(Xm);
fprintf('pca first eigenvalue = %.3f\n', latent(1));
fprintf('stats toolbox demo OK\n');
