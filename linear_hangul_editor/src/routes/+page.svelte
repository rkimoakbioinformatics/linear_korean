<script>
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { message } from "@tauri-apps/plugin-dialog";
    import { onMount } from "svelte";
    import GlyphDataEditor from "../lib/glyph_data_editor.svelte";
    import ConfigEditor from "$lib/config_editor.svelte";
    import KerningEditor from "$lib/kerning_editor.svelte";
    import { Menu, Submenu, PredefinedMenuItem } from "@tauri-apps/api/menu";

    let /** @type {any} */ kerning_editor_ref;
    let /** @type {any} */ config_editor_ref;
    let /** @type {any} */ glyph_data_editor_1;
    let /** @type {any} */ glyph_data_editor_2;
    let DEFAULT_FONT_NAME = "generated";
    let content = $state("");
    let char_size = $state("24");
    let char = $state("가");
    let content_name = $state("content.txt");
    let /** @type {any} */ content_names = $state([]);
    let fontname = $state(DEFAULT_FONT_NAME);
    let new_fontname = $state("");
    let /** @type {any} */ font_data = $state([]);
    let /** @type {any} */ fontnames = $state([]);
    let config_name = $state("default");
    let kerning_name = $state("default");
    let /** @type {any} */ loadedFont = null;
    let /** @type {any} */ glyph_set = $state("default");
    let /** @type {any} */ glyph_set_names = $state([]);
    let new_glyph_set = $state("");
    let /** @type {any} */ tool_set_names = $state([]);
    let tool_set_name = $state("");
    let new_tool_set_name = $state("");
    let error_msg = $state("");
    let ready_to_compile = $state(true);

    onMount(async function () {
        const copy = await PredefinedMenuItem.new({
            text: "copy-text",
            item: "Copy",
        });
        const cut = await PredefinedMenuItem.new({
            text: "cut-text",
            item: "Cut",
        });
        const paste = await PredefinedMenuItem.new({
            text: "paste-text",
            item: "Paste",
        });
        const undo = await PredefinedMenuItem.new({
            text: "undo",
            item: "Undo",
        });
        const redo = await PredefinedMenuItem.new({
            text: "redo",
            item: "Redo",
        });
        const select_all = await PredefinedMenuItem.new({
            text: "select-all",
            item: "SelectAll",
        });
        const refresh_content = {
            id: "refresh_content_list",
            text: "Refresh Content List",
            action: () => {
                console.log("refresh content pressed");
            },
        };
        let app_submenu = await Submenu.new({
            id: "edit",
            text: "Edit",
            items: [copy, cut, paste, undo, redo, select_all, refresh_content],
        });
        let menu = await Menu.new();
        menu.append(app_submenu);
        await menu.setAsAppMenu();
        await get_tool_set_names(null);
        await get_glyph_set_names(null);
        await get_font(null);
        await get_content_names(null);
        await get_content(null);
        await get_font_names(null);
    });

    const unlisten = listen("msg", async function (/** @type {any} */ event) {
        if (event.payload == "compile_ended") {
            fontname = "generated";
            await get_font(null);
            await get_font_names(event);
            ready_to_compile = true;
        }
    });
    const unlisten_error = listen(
        "error",
        async function (/** @type {any} */ event) {
            error_msg = event.payload;
            await message(error_msg, { title: "Error", kind: "error" });
            ready_to_compile = true;
        },
    );

    //
    // Toolset
    //
    /**
     * @param {any} event
     */
    async function get_tool_set_names(event) {
        tool_set_names = await invoke("get_tool_set_names", {});
    }

    /**
     * @param {any} event
     */
    async function get_tool_set_data(event) {
        let data = await invoke("get_tool_set_data", {
            toolSetName: tool_set_name,
        });
        glyph_set = data.glyph_set;
        config_name = data.config_name;
        kerning_name = data.kerning_name;
        await config_editor_ref.load_config_data(event);
        await kerning_editor_ref.load_kerning_data(event);
        await glyph_data_editor_1.loadGlyphDataWithoutSave(event);
        await glyph_data_editor_2.loadGlyphDataWithoutSave(event);
    }

    /**
     * @param {any} event
     */
    export async function save_tool_set(event) {
        if (new_tool_set_name == "") {
            return;
        }
        console.log("saving toolset", config_name, kerning_name, glyph_set);
        await invoke("save_tool_set", {
            toolSet: {
                config_name: config_name,
                kerning_name: kerning_name,
                glyph_set: glyph_set,
            },
            toolSetName: new_tool_set_name,
        });
        await get_tool_set_names(null);
        tool_set_name = new_tool_set_name;
    }

    /**
     * @param {any} event
     */
    /*export async function copy_tool_set(event) {
    if (tool_set_name == "" || new_tool_set_name == "") {
      return
    }
    await invoke("copy_tool_set", {
      toolSetName: tool_set_name,
      newToolSetName: new_tool_set_name
    });
    await get_tool_set_names(null)
    tool_set_name = new_tool_set_name
  }*/

    /**
     * @param {any} event
     */
    async function delete_tool_set(event) {
        if (glyph_set == "") {
            return;
        }
        await invoke("delete_tool_set", {
            toolSetName: tool_set_name,
        });
        await get_tool_set_names(null);
    }

    //
    // Glyph set
    /**
     * @param {any} event
     */
    async function get_glyph_set_names(event) {
        glyph_set_names = await invoke("get_glyph_set_names", {});
    }

    /**
     * @param {any} event
     */
    async function change_glyph_set(event) {
        console.log("change glyph_set:", glyph_set);
        await glyph_data_editor_1.loadGlyphDataWithoutSave(event);
        await glyph_data_editor_2.loadGlyphDataWithoutSave(event);
    }

    /**
     * @param {any} event
     */
    export async function copy_glyph_set(event) {
        if (glyph_set == "" || new_glyph_set == "") {
            return;
        }
        await invoke("copy_glyph_set", {
            glyphSet: glyph_set,
            newGlyphSet: new_glyph_set,
        });
        await get_glyph_set_names(null);
    }

    /**
     * @param {any} event
     */
    async function delete_glyph_set(event) {
        if (glyph_set == "") {
            return;
        }
        await invoke("delete_glyph_set", {
            glyphSet: glyph_set,
        });
        await get_glyph_set_names(null);
    }

    //
    // Font
    //
    /**
     * @param {any} event
     */
    async function get_font_names(event) {
        fontnames = await invoke("get_font_names", {});
    }

    /**
     * @param {any} event
     */
    async function get_font(event) {
        let data = await invoke("get_font_data", {
            fontName: fontname,
        });
        let /** @type{any} */ newFont = null;
        // Remove all prior faces for this family so browsers don't keep stale previews.
        for (let face of document.fonts) {
            if (face.family.replaceAll('"', "") == "Linear Korean") {
                document.fonts.delete(face);
            }
        }
        if (data.length > 0) {
            newFont = new FontFace("Linear Korean", new Uint8Array(data));
            await newFont.load();
        }
        if (newFont) {
            document.fonts.add(newFont);
            loadedFont = newFont;
        } else {
            loadedFont = null;
        }
    }

    /**
     * @param {any} event
     */
    async function delete_font(event) {
        await invoke("delete_font", {
            fontName: fontname,
        });
        await get_font_names(null);
    }

    /**
     * @param {any} event
     */
    async function save_font(event) {
        if (fontname == "" || new_fontname == "") {
            return;
        }
        invoke("save_font", {
            oldName: fontname,
            newName: new_fontname,
        })
            .then(() => {})
            .catch(async function (e) {
                await message(e, { title: "Error", kind: "error" });
            });
        await get_font_names(null);
        fontname = new_fontname;
    }

    //
    // Content
    //
    /**
     * @param {any} event
     */
    async function get_content_names(event) {
        content_names = await invoke("get_content_names", {});
    }

    /**
     * @param {any} event
     */
    async function get_content(event) {
        content = await invoke("get_content", {
            contentName: content_name,
        });
    }

    /**
     * @param {any} event
     */
    async function save_content(event) {
        await invoke("save_content", {
            content: content,
            contentName: content_name,
        });
    }

    /**
     * @param {any} event
     */
    async function compile(event) {
        if (glyph_set == "" || config_name == "" || kerning_name == "") {
            return;
        }
        ready_to_compile = false;
        await kerning_editor_ref.save(event);
        await config_editor_ref.save(event);
        await glyph_data_editor_1.save(event);
        await glyph_data_editor_2.save(event);
        await invoke("run_compile", {
            glyphSet: glyph_set,
            configName: config_name,
            kerningName: kerning_name,
        });
    }
</script>

<main class="w-full h-screen flex flex-col">
    <div class="flex justify-between h-96">
        <div class="flex items-center">
            <label for="char-size" class="">Size</label>
            <select bind:value={char_size}>
                <option value={16}>16</option>
                <option value={24}>24</option>
                <option value={100}>100</option>
            </select>
            <select bind:value={content_name} onchange={get_content}>
                {#each content_names as item}
                    <option value={item}>{item}</option>
                {/each}
            </select>
            <button onclick={get_content} class="mx-1">Load</button>
            <button onclick={save_content} class="mx-1">Save</button>
            <button onclick={get_content_names} class="mx-1">Refresh</button>
            <div class="mx-4">
                <input
                    type="text"
                    bind:value={char}
                    class="border-1 w-8 self-center p-2"
                />
                <span class="">{char}</span>
                <span class="linkor">{char}</span>
            </div>
            <select bind:value={fontname} onchange={get_font}>
                {#each fontnames as item}
                    <option value={item}>{item}</option>
                {/each}
            </select>
            <button onclick={delete_font} class="mx-1">Delete</button>
            <button onclick={get_font_names} class="mx-1">Refresh</button>
            <input
                type="text"
                bind:value={new_fontname}
                class="border rounded-md p-2"
                placeholder="Font name"
            />
            <button onclick={save_font}>Save Font</button>
        </div>
        <div>
            <button
                onclick={compile}
                class={["text-xl p-2 w-48", !ready_to_compile && "disabled"]}
                disabled={!ready_to_compile}
            >
                {#if ready_to_compile}
                    Compile
                {:else}
                    Compiling...
                {/if}
            </button>
        </div>
    </div>
    <div class="flex space-x-4 h-full">
        <div class="flex flex-col w-6/10 space-y-0">
            <textarea
                class="border rounded-md h-[40%] p-2"
                bind:value={content}
                style:font-size={char_size + "px"}
            ></textarea>
            <textarea
                class="border rounded-md linkor h-[40%] p-2"
                bind:value={content}
                style:font-size={char_size + "px"}
            ></textarea>
        </div>
        <div class="flex flex-col space-y-4 w-4/10 h-full">
            <div class="flex">
                Toolset
                <select bind:value={tool_set_name} onchange={get_tool_set_data}>
                    {#each tool_set_names as item}
                        <option value={item}>{item}</option>
                    {/each}
                </select>
                <button onclick={delete_tool_set} class="mx-1">Delete</button>
                <input
                    type="text"
                    bind:value={new_tool_set_name}
                    placeholder="Tool set"
                    onchange={get_tool_set_data}
                    class="border rounded-md p-1"
                />
                <button onclick={save_tool_set} class="mx-1">Save</button>
                <!--<button onclick={copy_tool_set} class="mx-1">Copy</button>-->
            </div>
            <div class="flex space-y-4">
                <KerningEditor bind:this={kerning_editor_ref} bind:kerning_name
                ></KerningEditor>
                <ConfigEditor bind:this={config_editor_ref} bind:config_name
                ></ConfigEditor>
            </div>
            <div class="flex flex-col space-y-4">
                <div class="flex">
                    <select bind:value={glyph_set} onchange={change_glyph_set}>
                        {#each glyph_set_names as item}
                            <option value={item}>{item}</option>
                        {/each}
                    </select>
                    <button onclick={get_glyph_set_names} class="mx-1"
                        >Refresh</button
                    >
                    <button onclick={delete_glyph_set} class="mx-1"
                        >Delete</button
                    >
                    <input
                        type="text"
                        bind:value={new_glyph_set}
                        placeholder="Glyph set"
                        onchange={copy_glyph_set}
                        class="border rounded-md p-1"
                    />
                    <button onclick={copy_glyph_set} class="mx-1">Copy</button>
                </div>
                <GlyphDataEditor bind:this={glyph_data_editor_1} {glyph_set}
                ></GlyphDataEditor>
                <GlyphDataEditor bind:this={glyph_data_editor_2} {glyph_set}
                ></GlyphDataEditor>
            </div>
        </div>
    </div>
</main>

<style lang="postcss">
    @import "tailwindcss";

    @font-face {
        font-family: "Linear Korean";
        src: url("");
    }

    select {
        @apply text-lg p-2;
    }

    button {
        @apply rounded-sm bg-indigo-600 px-2 py-1 text-base font-semibold text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600;
    }

    button.disabled {
        @apply bg-stone-500;
    }

    .linkor {
        font-family: "Linear Korean";
    }
</style>
