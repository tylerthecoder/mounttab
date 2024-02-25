import type { TabUrl, WindowId } from "./types"
import { $ } from "bun";

export type TabState = {
    tabs: Record<WindowId, TabUrl[]>
}

export const getTabStateFromFs = async (): Promise<TabState> => {
    const STATE_FILE = `${process.env.HOME}/.config/mt/state.json`;
    await $`mkdir -p ${process.env.HOME}/.config/mt`;

    let currentTabState = await (async () => {
        const file = Bun.file(STATE_FILE)
        if (!(await file.exists())) {
            return {
                tabs: {}
            } as TabState;
        }

        return await file.json() as TabState;
    })();

    return currentTabState;
}

export const saveTabStateToFs = async (state: TabState) => {
    const STATE_FILE = `${process.env.HOME}/.config/mt/state.json`;
    await $`mkdir -p ${process.env.HOME}/.config/mt`;
    await Bun.write(STATE_FILE, JSON.stringify(state, null, 2));
}
