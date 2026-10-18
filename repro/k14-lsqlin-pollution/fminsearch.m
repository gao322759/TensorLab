function [x, f] = fminsearch(fun, x0)
%FMINSEARCH K14 最小受害者（只保留一次函数句柄调用，不是完整的 Nelder–Mead）。
%   与 lsqlin.m 同目录并 addpath 后，下面的 fun(x) 会错位并报
%   "Not enough input arguments."；单独放在目录里则正常。
    x = x0;
    f = fun(x);
end
