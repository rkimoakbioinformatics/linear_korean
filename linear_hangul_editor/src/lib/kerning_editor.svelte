<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    let kerning_data = $state("");
    let /** @type {any} */ kerning_names = $state([]);
    let { kerning_name = $bindable() } = $props();

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
     * @param {string | null} [next_kerning_name]
     */
    export async function save(event, next_kerning_name = null) {
        let target_kerning_name =
            next_kerning_name == null ? kerning_name : next_kerning_name.trim();
        if (target_kerning_name == "") {
            return;
        }
        await invoke("save_kerning_data", {
            kerningData: kerning_data,
            kerningName: target_kerning_name,
        });
        kerning_name = target_kerning_name;
        await getKerningNames(null);
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
    </div>
    <textarea class="w-full h-64 p-2 border-1" bind:value={kerning_data}
    ></textarea>
</div>

<style lang="postcss">
    @import "tailwindcss";
    button {
        @apply rounded-sm px-2 py-1 text-xs font-semibold text-white shadow-xs focus-visible:outline-2 focus-visible:outline-offset-2;
    }
</style>
