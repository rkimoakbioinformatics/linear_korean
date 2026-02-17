<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    let config_data = $state("");
    let { config_name = $bindable() } = $props();
    let /** @type {any} */ config_names = $state([]);
    let new_config_name = $state("");

    onMount(async function () {
        await get_config_names(null);
        await load_config_data(null);
    });

    /**
     * @param {any} event
     */
    async function get_config_names(event) {
        config_names = await invoke("get_config_names", {});
    }

    /**
     * @param {any} event
     */
    export async function load_config_data(event) {
        config_data = await invoke("get_config_data", {
            configName: config_name,
        });
        new_config_name = config_name;
    }

    /**
     * @param {any} event
     */
    export async function save(event) {
        if (new_config_name == "") {
            return;
        }
        await invoke("save_config", {
            configData: config_data,
            configName: config_name,
        });
        await get_config_names(null);
        config_name = new_config_name;
    }
</script>

<div class="">
    <div>
        <select bind:value={config_name} onchange={load_config_data}>
            {#each config_names as item}
                <option value={item}>{item}</option>
            {/each}
        </select>
        <button onclick={load_config_data}>&#x1F504;</button>
        <input
            type="text"
            bind:value={config_name}
            placeholder="Config name"
            onchange={save}
            class="border rounded-md p-1"
        />
        <button onclick={save}>&#x1F4BE;</button>
    </div>
    <textarea class="w-full h-64 p-2 border-1" bind:value={config_data}
    ></textarea>
</div>

<style lang="postcss">
    @import "tailwindcss";
    button {
        @apply rounded-sm px-2 py-1 text-xs font-semibold text-white shadow-xs focus-visible:outline-2 focus-visible:outline-offset-2;
    }
</style>
