%% Image Processing toolbox demo (v0.2)
% 小图（6×6）：bwdist 是 O(W*H*F) 精确搜索，测试只关心语义正确，不追求分辨率。
n = 6;
I = zeros(n, n);
for i = 1:n
    for j = 1:n
        I(i, j) = 0.5 + 0.4 * sin(2 * pi * i / 8) * cos(2 * pi * j / 8);
    end
end
for i = 2:4
    for j = 2:4
        I(i, j) = 0.95;
    end
end

J = mat2gray(I);
fprintf('mat2gray range = %.2f %.2f\n', min(min(J)), max(max(J)));

S = imresize(J, 0.5, 'bilinear');
fprintf('imresize size = %d x %d\n', size(S, 1), size(S, 2));

M = medfilt2(J, [3 3]);
fprintf('medfilt2 center = %.3f\n', M(3, 3));

E = edge(J, 'sobel');
fprintf('edge pixels = %d\n', sum(sum(E)));

BW = zeros(n, n);
for i = 2:4
    for j = 2:4
        BW(i, j) = 1;
    end
end
BW2 = imdilate(BW, ones(3, 3));
fprintf('imdilate area = %d\n', sum(sum(BW2)));

[L, nl] = bwlabel(BW2);
fprintf('bwlabel regions = %d\n', nl);

D = bwdist(BW);
% 取一个背景角点，确保测试的是“到最近前景的距离”，而不是前景自身的 0。
fprintf('bwdist corner = %.2f\n', D(1, 1));

H = histeq(J);
fprintf('histeq center = %.3f\n', H(3, 3));

R = imrotate(J, 30, 'bilinear');
fprintf('imrotate size = %d x %d\n', size(R, 1), size(R, 2));

C = imcrop(J, [1 1 4 4]);
fprintf('imcrop size = %d x %d\n', size(C, 1), size(C, 2));

Q = imquantize(J, [0.25 0.5 0.75]);
fprintf('imquantize levels = %d\n', max(max(Q)));
fprintf('image toolbox demo OK\n');
