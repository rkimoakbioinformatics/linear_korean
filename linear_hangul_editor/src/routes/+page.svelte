<script>
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
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
    let char_size = $state("16");
    let char = $state("가");
    let content_name = $state("content.txt");
    let /** @type {any} */ content_names = $state([]);
    let fontname = $state(DEFAULT_FONT_NAME);
    let /** @type {any} */ font_data = $state([]);
    let /** @type {any} */ fontnames = $state([]);
    let config_name = $state("default");
    let kerning_name = $state("default");
    let /** @type {any} */ loadedFont = null;
    let /** @type {any} */ glyph_set = $state("default");
    let /** @type {any} */ glyph_set_names = $state([]);
    let /** @type {any} */ tool_set_names = $state([]);
    let tool_set_name = $state("");
    let error_msg = $state("");
    let ready_to_compile = $state(true);
    let save_prompt_open = $state(false);
    let save_prompt_title = $state("");
    let save_prompt_value = $state("");
    let /** @type {((value: string | null) => void) | null} */ save_prompt_resolve =
            null;
    let alert_dialog_open = $state(false);
    let alert_dialog_title = $state("Error");
    let alert_dialog_message = $state("");

    /**
     * @param {any} error
     * @returns {string}
     */
    function format_error_message(error) {
        if (error == null) {
            return "Unknown error";
        }
        if (typeof error == "string") {
            return error;
        }
        if (error instanceof Error) {
            return error.message;
        }
        try {
            return JSON.stringify(error, null, 2);
        } catch {
            return String(error);
        }
    }

    /**
     * @param {string} title
     * @param {string} message_text
     */
    function open_alert_dialog(title, message_text) {
        alert_dialog_title = title;
        alert_dialog_message = message_text
            .replaceAll("\\r\\n", "\n")
            .replaceAll("\\n", "\n")
            .replaceAll("\\r", "\n");
        alert_dialog_open = true;
    }

    function close_alert_dialog() {
        alert_dialog_open = false;
    }

    /**
     * @param {string | null} value
     */
    function close_save_prompt(value) {
        if (save_prompt_resolve) {
            save_prompt_resolve(value);
        }
        save_prompt_resolve = null;
        save_prompt_open = false;
        save_prompt_title = "";
        save_prompt_value = "";
    }

    /**
     * @param {string} title
     * @param {string} initial_value
     * @returns {Promise<string | null>}
     */
    function request_save_name(title, initial_value) {
        save_prompt_title = title;
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
            error_msg = format_error_message(event.payload);
            open_alert_dialog("Compile Error", error_msg);
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
        await config_editor_ref.save(event);
        await kerning_editor_ref.save(event);
        await glyph_data_editor_1.save(event);
        await glyph_data_editor_2.save(event);
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
        let next_tool_set_name = await request_save_name(
            "Save tool set as",
            tool_set_name || "",
        );
        if (next_tool_set_name == null) {
            return;
        }
        next_tool_set_name = next_tool_set_name.trim();
        if (next_tool_set_name == "") {
            return;
        }
        await glyph_data_editor_1.save(event);
        await glyph_data_editor_2.save(event);
        await config_editor_ref.save(event, next_tool_set_name);
        await kerning_editor_ref.save(event, next_tool_set_name);
        if (glyph_set != next_tool_set_name) {
            await invoke("copy_glyph_set", {
                glyphSet: glyph_set,
                newGlyphSet: next_tool_set_name,
            });
        }
        await get_glyph_set_names(null);
        glyph_set = next_tool_set_name;
        config_name = next_tool_set_name;
        kerning_name = next_tool_set_name;
        await invoke("save_tool_set", {
            toolSet: {
                config_name: next_tool_set_name,
                kerning_name: next_tool_set_name,
                glyph_set: next_tool_set_name,
            },
            toolSetName: next_tool_set_name,
        });
        await get_tool_set_names(null);
        tool_set_name = next_tool_set_name;
        await glyph_data_editor_1.loadGlyphDataWithoutSave(event);
        await glyph_data_editor_2.loadGlyphDataWithoutSave(event);
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
        await glyph_data_editor_1.save(event);
        await glyph_data_editor_2.save(event);
        await glyph_data_editor_1.loadGlyphDataWithoutSave(event);
        await glyph_data_editor_2.loadGlyphDataWithoutSave(event);
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
        if (fontname == "") {
            return;
        }
        let next_font_name = await request_save_name(
            "Save font as",
            fontname || "",
        );
        if (next_font_name == null) {
            return;
        }
        invoke("save_font", {
            oldName: fontname,
            newName: next_font_name,
        })
            .then(() => {})
            .catch(async function (e) {
                error_msg = format_error_message(e);
                open_alert_dialog("Error", error_msg);
            });
        await get_font_names(null);
        fontname = next_font_name;
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
        let next_content_name = await request_save_name(
            "Save content as filename",
            content_name || "",
        );
        if (next_content_name == null) {
            return;
        }
        await invoke("save_content", {
            content: content,
            contentName: next_content_name,
        });
        content_name = next_content_name;
        await get_content_names(null);
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
        try {
            await invoke("run_compile", {
                glyphSet: glyph_set,
                configName: config_name,
                kerningName: kerning_name,
            });
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Compile Error", error_msg);
            ready_to_compile = true;
        }
    }
</script>

<main class="w-full h-screen flex flex-col">
    <div class="flex items-center">
        <div class="border-r">
            <label for="char-size">Size</label>
            <select id="char-size" bind:value={char_size}>
                <option value={16}>16</option>
                <option value={24}>24</option>
                <option value={48}>48</option>
                <option value={64}>64</option>
                <option value={100}>100</option>
            </select>
            <select bind:value={content_name} onchange={get_content}>
                {#each content_names as item}
                    <option value={item}>{item}</option>
                {/each}
            </select>
            <button onclick={get_content_names} class="nobg">&#x1F504;</button>
            <button
                onclick={get_content}
                class="nobg"
                title="Load content"
                aria-label="Load content"
            >
                &#x1F4C2;
            </button>
            <button
                onclick={save_content}
                class="nobg"
                title="Save content"
                aria-label="Save content"
            >
                &#x1F4BE;
            </button>
        </div>
        <div class="border-r">
            <div class="mx-4">
                <input type="text" bind:value={char} class="w-4" />
                <span>&#x2192;</span>
                <span class="linkor w-4">{char}</span>
            </div>
        </div>
        <div class="border-r">
            <select bind:value={fontname} onchange={get_font}>
                {#each fontnames as item}
                    <option value={item}>{item}</option>
                {/each}
            </select>
            <button
                onclick={delete_font}
                class="mx-1 nobg"
                title="Delete font"
                aria-label="Delete font"
            >
                &#x1F5D1;
            </button>
            <button
                onclick={get_font_names}
                class="mx-1 nobg"
                title="Refresh fonts"
                aria-label="Refresh fonts"
            >
                &#x1F504;
            </button>
            <button
                class="nobg"
                onclick={save_font}
                title="Save font as"
                aria-label="Save font as"
            >
                &#x1F4BE;
            </button>
        </div>
        <div class="ml-2">
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
        <div class="flex flex-col grow space-y-0">
            <textarea
                class="border rounded-md linkor grow p-2"
                bind:value={content}
                style:font-size={char_size + "px"}
            ></textarea>
            <textarea
                class="border rounded-md h-48 p-2"
                bind:value={content}
                style="font-size=16px;"
            ></textarea>
        </div>
        <div class="flex flex-col space-y-4 w-96 h-full">
            <div class="flex">
                Toolset
                <select bind:value={tool_set_name} onchange={get_tool_set_data}>
                    {#each tool_set_names as item}
                        <option value={item}>{item}</option>
                    {/each}
                </select>
                <button
                    onclick={delete_tool_set}
                    class="mx-1 nobg"
                    title="Delete tool set"
                    aria-label="Delete tool set"
                >
                    &#x1F5D1;
                </button>
                <button
                    onclick={save_tool_set}
                    class="mx-1 nobg"
                    title="Save tool set as"
                    aria-label="Save tool set as"
                >
                    &#x1F4BE;
                </button>
                <!--<button onclick={copy_tool_set} class="mx-1">Copy</button>-->
            </div>
            <div class="flex space-y-4">
                <KerningEditor bind:this={kerning_editor_ref} bind:kerning_name
                ></KerningEditor>
                <ConfigEditor bind:this={config_editor_ref} bind:config_name
                ></ConfigEditor>
            </div>
            <div class="flex flex-col">
                <div class="flex items-center">
                    <span>Glyph Set</span>
                    <select bind:value={glyph_set} onchange={change_glyph_set}>
                        {#each glyph_set_names as item}
                            <option value={item}>{item}</option>
                        {/each}
                    </select>
                    <button
                        onclick={get_glyph_set_names}
                        class="mx-1 nobg"
                        title="Refresh glyph sets"
                        aria-label="Refresh glyph sets"
                    >
                        &#x1F504;
                    </button>
                    <button
                        onclick={delete_glyph_set}
                        class="mx-1 nobg"
                        title="Delete glyph set"
                        aria-label="Delete glyph set"
                    >
                        &#x1F5D1;
                    </button>
                </div>
                <div class="flex flex-col">
                    <GlyphDataEditor bind:this={glyph_data_editor_1} {glyph_set}
                    ></GlyphDataEditor>
                    <GlyphDataEditor bind:this={glyph_data_editor_2} {glyph_set}
                    ></GlyphDataEditor>
                </div>
            </div>
        </div>
    </div>
    {#if save_prompt_open}
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        >
            <div class="w-[26rem] rounded-md bg-stone-900 p-4 text-white">
                <div class="text-sm font-semibold">{save_prompt_title}</div>
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
                    <button
                        class="nobg"
                        onclick={() => close_save_prompt(null)}
                    >
                        Cancel
                    </button>
                    <button onclick={confirm_save_prompt}>Save</button>
                </div>
            </div>
        </div>
    {/if}
    {#if alert_dialog_open}
        <div
            class="fixed inset-0 z-[60] flex items-center justify-center bg-black/50"
        >
            <div class="w-[32rem] rounded-md bg-stone-900 p-4 text-white">
                <div class="text-sm font-semibold">{alert_dialog_title}</div>
                <pre
                    class="mt-3 max-h-72 overflow-auto whitespace-pre-wrap rounded-md border border-stone-600 bg-stone-800 p-2 text-sm">{alert_dialog_message}</pre>
                <div class="mt-3 flex justify-end">
                    <button onclick={close_alert_dialog}>OK</button>
                </div>
            </div>
        </div>
    {/if}
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
        @apply rounded-sm bg-blue-600 px-2 py-1 text-base font-semibold text-white shadow-xs hover:bg-blue-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500;
    }

    button.disabled {
        @apply bg-stone-500;
    }

    button.nobg {
        @apply rounded-sm bg-transparent px-2 py-1 text-base font-semibold text-white shadow-xs hover:bg-transparent focus-visible:outline-2 focus-visible:outline-offset-2;
    }

    .linkor {
        font-family: "Linear Korean";
    }
</style>
