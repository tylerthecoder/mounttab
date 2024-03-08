
type Config = {
    stateFile: string;
}

const DEFAULT_CONFIG: Config = {
    stateFile: `${process.env.HOME}/.config/mt/state.json`,
}

export const getConfig = async (): Promise<Config> => {
    const { $ } = await import("bun");
    const toml = await import("toml");

    const configFilePath = `${process.env.HOME}/.config/mt/mt.toml`;
    await $`mkdir -p ${process.env.HOME}/.config/mt`;

    const configFile = Bun.file(configFilePath);

    if (!(await configFile.exists())) {
        return DEFAULT_CONFIG;
    }

    const config = toml.parse(await configFile.text()) as Record<string, unknown>;

    if (!config || typeof config !== "object") {
        console.log("Invalid config file");
        return DEFAULT_CONFIG;
    }

    function getProp(prop: keyof Config) {
        if (!(prop in config)) {
            return DEFAULT_CONFIG[prop];
        }
        return config[prop];
    }

    function getStringProp(prop: keyof Config): string {
        const val = getProp(prop);
        return String(val);
    }

    const stateFile = getStringProp("stateFile");

    return {
        stateFile,
    }
}


