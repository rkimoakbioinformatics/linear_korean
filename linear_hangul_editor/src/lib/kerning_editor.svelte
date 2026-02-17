<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  let kerning_data = $state("");
  let /** @type {any} */ kerning_names = $state([]);
  let { kerning_name = $bindable() } = $props();
  let new_kerning_name = $state("")

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
    new_kerning_name = kerning_name
  }

  /**
   * @param {any} event
   */
  export async function save(event) {
    if (new_kerning_name == "") {
      return;
    }
    await invoke("save_kerning_data", {
      kerningData: kerning_data,
      kerningName: new_kerning_name,
    });
    await getKerningNames(null)
  }
</script>

<div class="">
  <div>
    <button onclick={load_kerning_data}>Reload</button>
    <select bind:value={kerning_name} onchange={load_kerning_data}>
      {#each kerning_names as item}
        <option value={item}>{item}</option>
      {/each}
    </select>
    <input
      type="text"
      bind:value={new_kerning_name}
      placeholder="Kerning name"
      onchange={save}
      class="border rounded-md p-1"
    />
    <button onclick={save}>Save</button>
  </div>
  <textarea class="w-full h-64 p-2 border-1" bind:value={kerning_data}
  ></textarea>
</div>

<style lang="postcss">
  @import "tailwindcss";
  button {
    @apply rounded-sm bg-indigo-600 px-2 py-1 text-xs font-semibold text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600;
  }
</style>
