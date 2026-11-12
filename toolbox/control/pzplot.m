function pzplot(sys)
%PZPLOT Pole-zero map of a SISO LTI model.
%   PZPLOT(SYS) draws the poles as 'x' and the zeros as 'o' in the complex
%   plane.  Data extraction is delegated to PZMAP; this function is the
%   plotting wrapper.
%
%   MATLAB's PZPLOT returns a chart object when an output is requested;
%   TensorLab does not model chart objects, so use [P,Z] = PZMAP(SYS) for
%   the numeric data.
    arguments
        sys
    end
    [p, z] = pzmap(sys);
    plot(real(p), imag(p), 'xr');
    hold on;
    plot(real(z), imag(z), 'ob');
    hold off;
    grid on;
    title('Pole-Zero Map');
    xlabel('Real Axis');
    ylabel('Imaginary Axis');
end
