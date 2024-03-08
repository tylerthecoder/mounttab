import { $ } from "bun";
import toml from "toml";

type Config = {
    stateFile: string;
    serverPort: number;
}

const DEFAULT_CONFIG: Config = {
    stateFile: `${process.env.HOME}/.config/mt/state.json`,
    serverPort: 3149
}

export const getConfig = async (): Promise<Config> => {
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

    function getNumberProp(prop: keyof Config): number {
        const val = getProp(prop);
        return parseInt(String(val), 10);
    }


    const stateFile = getStringProp("stateFile");
    const serverPort = getNumberProp("serverPort");

    return {
        stateFile,
        serverPort,
    }
}


