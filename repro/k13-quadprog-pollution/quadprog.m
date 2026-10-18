function x = quadprog(H, f)
%QUADPROG K13 最小污染样本（stub，不是完整的二次规划实现）。
%   本文件只保留触发条件：一个公开函数 + 一个同文件局部函数。
%   与 fminsearch.m 同目录并 addpath 后，fminsearch 的函数句柄调用会错位，
%   报 "Not enough input arguments."。
%   v0.2 的完整实现（工作集循环 + 乘子下降）触发了同一问题，见 docs/kernel-notes.md 坑 25。
    x = qp_core(H, f);
end

function x = qp_core(H, f)
    x = H \ f;
end
