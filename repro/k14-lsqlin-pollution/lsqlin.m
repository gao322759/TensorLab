function x = lsqlin(C, d)
%LSQLIN K14 最小污染样本（stub，不是完整的有约束最小二乘实现）。
%   与 fminsearch.m 同目录并 addpath 后，fminsearch 的函数句柄调用会错位，
%   报 "Not enough input arguments."。
%   v0.2 观测到的另一种表现：lsqlin 与 lsqnonneg 同目录时，调用被错误绑定到
%   lsqnonneg（"semantic function lsqnonneg expected 2 inputs, got 9"）。
%   见 docs/kernel-notes.md 坑 26。
    x = lsq_core(C, d);
end

function x = lsq_core(C, d)
    x = C \ d;
end
