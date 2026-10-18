function matlab_bench()
%MATLAB_BENCH Comparison workloads for `tensorlab bench`.
%   在 MATLAB（R2023b / R2026a）中运行；与 TensorLab bench 使用同一组 workload。
%   前 10 次预热、后 5 次取中位数，和 CLI bench 的协议一致。
%   循环上界带一个 rand 依赖，避免 MATLAB JIT 把纯常量循环折叠掉。
    warmup = 10;
    reps = 5;
    names = {'script_loop_5k', 'function_calls_100', 'matmul_300', ...
             'fft_16k', 'vector_math_100k'};
    fprintf('MATLAB benchmark (%s)\n', version('-release'));
    fprintf('%-20s %10s %10s %8s\n', 'workload', 'cold_ms', 'steady_ms', 'speedup');
    for w = 1:numel(names)
        name = names{w};
        cold = 0;
        steady = zeros(1, reps);
        for k = 1:(warmup + reps)
            t0 = tic;
            run_workload(name);
            dt = toc(t0) * 1000;
            if k == 1
                cold = dt;
            end
            if k > warmup
                steady(k - warmup) = dt;
            end
        end
        s = median(steady);
        fprintf('%-20s %10.3f %10.3f %7.2fx\n', name, cold, s, cold / s);
    end
end

function run_workload(name)
    switch name
        case 'script_loop_5k'
            n = 5000 + (rand() > 2);
            benchmark = 0;
            for i = 1:n
                benchmark = benchmark + i;
            end
            assert(benchmark > 0);
        case 'function_calls_100'
            m = 50 + (rand() > 2);
            benchmark = 0;
            for k = 1:100
                benchmark = benchmark + tl_bench_loop(m);
            end
            assert(benchmark > 0);
        case 'matmul_300'
            A = ones(300);
            B = ones(300);
            for k = 1:5
                C = A * B;
            end
            assert(C(1, 1) == 300);
        case 'fft_16k'
            x = ones(1, 16384);
            for k = 1:20
                y = fft(x);
            end
            assert(numel(y) == 16384);
        case 'vector_math_100k'
            x = (1:100000);
            for k = 1:10
                y = sin(x) + cos(x);
            end
            assert(numel(y) == 100000);
    end
end

function s = tl_bench_loop(n)
    s = 0;
    for i = 1:n
        s = s + i;
    end
end
