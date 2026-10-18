function val = optimget(opts, name, default)
    arguments
        opts
        name
        default = []
    end
    val = default;
    try
        val = getfield(opts, name);
    catch
        val = default;
    end
end
