<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    let kerning_data = $state("");
    let /** @type {any} */ kerning_names = $state([]);
    let { kerning_name = $bindable() } = $props();
    let save_prompt_open = $state(false);
    let save_prompt_value = $state("");
    let /** @type {((value: string | null) => void) | null} */ save_prompt_resolve =
            null;

    /**
     * @param {string | null} value
     */
    function close_save_prompt(value) {
        if (save_prompt_resolve) {
            save_prompt_resolve(value);
        }
        save_prompt_resolve = null;
        save_prompt_open = false;
        save_prompt_value = "";
    }

    /**
     * @param {string} initial_value
     * @returns {Promise<string | null>}
     */
    function request_save_name(initial_value) {
        save_prompt_value = initial_value;
        save_prompt_open = true;
        return new Promise((resolve) => {
            save_prompt_resolve = resolve;
        });
    }

    function confirm_save_prompt() {
        const next_value = save_prompt_value.trim();
        if (next_value == "") {
            return;
        }
        close_save_prompt(next_value);
    }

    onMount(async function () {
        await getKerningNames(null);
        await load_kerning_data(null);
    });

    /**
     * @param {any} event
     */
    async function getKerningNames(event) {
        kerning_names = await invoke("get_kerning_names", {});
    }

    /**
     * @param {any} event
     */
    export async function load_kerning_data(event) {
        kerning_data = await invoke("get_kerning_data", {
            kerningName: kerning_name,
        });
    }

    /**
     * @param {any} event
     */
    export async function save(event) {
        if (kerning_name == "") {
            return;
        }
        await invoke("save_kerning_data", {
            kerningData: kerning_data,
            kerningName: kerning_name,
        });
        await getKerningNames(null);
    }

    /**
     * @param {any} event
     */
    async function save_with_prompt(event) {
        let next_kerning_name = await request_save_name(kerning_name || "");
        if (next_kerning_name == null) {
            return;
        }
        kerning_name = next_kerning_name;
        await save(event);
    }
</script>

<div class="">
    <div>
        <select bind:value={kerning_name} onchange={load_kerning_data}>
            {#each kerning_names as item}
                <option value={item}>{item}</option>
            {/each}
        </select>
        <button onclick={load_kerning_data}>&#x1F504;</button>
        <button
            onclick={save_with_prompt}
            title="Save kerning as"
            aria-label="Save kerning as"
        >
            &#x1F4BE;
        </button>
    </div>
    <textarea class="w-full h-64 p-2 border-1" bind:value={kerning_data}
    ></textarea>
    {#if save_prompt_open}
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        >
            <div class="w-[24rem] rounded-md bg-stone-900 p-4 text-white">
                <div class="text-sm font-semibold">Save kerning as</div>
                <input
                    class="mt-3 w-full rounded-md border border-stone-600 bg-stone-800 p-2 text-base"
                    bind:value={save_prompt_value}
                    onkeydown={(event) => {
                        if (event.key == "Enter") {
                            confirm_save_prompt();
                        } else if (event.key == "Escape") {
                            close_save_prompt(null);
                        }
                    }}
                />
                <div class="mt-3 flex justify-end gap-2">
                    <button onclick={() => close_save_prompt(null)}
                        >Cancel</button
                    >
                    <button onclick={confirm_save_prompt}>&#x1F4BE;</button>
                </div>
            </div>
        </div>
    {/if}
</div>

<style lang="postcss">
    @import "tailwindcss";
    button {
        @apply rounded-sm px-2 py-1 text-xs font-semibold text-white shadow-xs focus-visible:outline-2 focus-visible:outline-offset-2;
    }
</style>
