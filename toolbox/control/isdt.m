function flag = isdt(sys)
%ISDT True for discrete-time models.
%   FLAG = ISDT(SYS) returns 1 if SYS has a non-zero sample time.
    arguments
        sys
    end
    if sys.Ts == 0
        flag = 0;
    else
        flag = 1;
    end
end
