function sysd = d2d(sys, TsOld, TsNew, method)
%D2D Resample a discrete transfer function.
%   SYSD = D2D(SYS,TSOLD,TSNEW) converts SYS (sampled at TSOLD) to the new
%   sampling period TSNEW using D2C followed by C2D (Tustin).
%   MATLAB reads the old sampling time from the model; this kernel build
%   requires it explicitly.
    arguments
        sys
        TsOld double
        TsNew double
        method = 'tustin'
    end
    sysc = d2c(sys, TsOld, method);
    sysd = c2d(sysc, TsNew, method);
end
