function flag = isct(sys)
%ISCT True for continuous-time models.
%   FLAG = ISCT(SYS) returns 1 if SYS has sample time 0.
    arguments
        sys
    end
    if sys.Ts == 0
        flag = 1;
    else
        flag = 0;
    end
end
