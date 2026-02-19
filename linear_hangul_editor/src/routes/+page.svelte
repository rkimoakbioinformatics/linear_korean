<script>
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onMount } from "svelte";
    import { Menu, Submenu, PredefinedMenuItem } from "@tauri-apps/api/menu";
    import {
        FolderOpen,
        Hammer,
        Moon,
        RefreshCcw,
        Save,
        Sun,
        Trash2,
    } from "lucide-svelte";

    import GlyphDataEditor from "$lib/glyph_data_editor.svelte";
    import ConfigEditor from "$lib/config_editor.svelte";
    import KerningEditor from "$lib/kerning_editor.svelte";

    let kerning_editor_ref = $state(null);
    let config_editor_ref = $state(null);
    let glyph_data_editor_1 = $state(null);
    let glyph_data_editor_2 = $state(null);

    let DEFAULT_FONT_NAME = "generated";
    let content = $state("");
    let char_size = $state("16");
    let char = $state("가");
    let content_name = $state("content.txt");
    let content_names = $state([]);
    let fontname = $state(DEFAULT_FONT_NAME);
    let fontnames = $state([]);
    let config_name = $state("default");
    let kerning_name = $state("default");
    let glyph_set = $state("default");
    let glyph_set_names = $state([]);
    let tool_set_names = $state([]);
    let tool_set_name = $state("");

    let error_msg = $state("");
    let ready_to_compile = $state(true);

    let save_prompt_open = $state(false);
    let save_prompt_title = $state("");
    let save_prompt_value = $state("");
    let save_prompt_resolve = null;

    let alert_dialog_open = $state(false);
    let alert_dialog_title = $state("Error");
    let alert_dialog_message = $state("");

    let theme_mode = $state("light");
    let should_persist_ui_state = false;

    const UI_SETTING_KEYS = {
        char_size: "char_size",
        content_name: "content_name",
        char: "char",
        fontname: "fontname",
        tool_set_name: "tool_set_name",
        theme_mode: "theme_mode",
    };

    function apply_theme_mode(next_theme_mode) {
        if (typeof document == "undefined") {
            return;
        }
        document.documentElement.classList.toggle(
            "dark",
            next_theme_mode == "dark",
        );
    }

    function toggle_theme_mode() {
        theme_mode = theme_mode == "dark" ? "light" : "dark";
        apply_theme_mode(theme_mode);
    }

    async function get_setting_or_default(name, fallback) {
        try {
            let value = await invoke("get_setting", { name });
            return value == null ? fallback : value;
        } catch (e) {
            console.warn(`get_setting failed for ${name}`, e);
            return fallback;
        }
    }

    async function save_ui_setting(name, value) {
        if (!should_persist_ui_state) {
            return;
        }
        try {
            await invoke("set_setting", { name, value });
        } catch (e) {
            console.warn(`set_setting failed for ${name}`, e);
        }
    }

    async function load_persisted_ui_state() {
        char_size = await get_setting_or_default(
            UI_SETTING_KEYS.char_size,
            char_size,
        );
        content_name = await get_setting_or_default(
            UI_SETTING_KEYS.content_name,
            content_name,
        );
        char = await get_setting_or_default(UI_SETTING_KEYS.char, char);
        fontname = await get_setting_or_default(
            UI_SETTING_KEYS.fontname,
            fontname,
        );
        tool_set_name = await get_setting_or_default(
            UI_SETTING_KEYS.tool_set_name,
            tool_set_name,
        );
        theme_mode = await get_setting_or_default(
            UI_SETTING_KEYS.theme_mode,
            theme_mode,
        );
    }

    $effect(() => {
        void save_ui_setting(UI_SETTING_KEYS.char_size, String(char_size));
    });

    $effect(() => {
        void save_ui_setting(UI_SETTING_KEYS.content_name, String(content_name));
    });

    $effect(() => {
        void save_ui_setting(UI_SETTING_KEYS.char, String(char));
    });

    $effect(() => {
        void save_ui_setting(UI_SETTING_KEYS.fontname, String(fontname));
    });

    $effect(() => {
        void save_ui_setting(UI_SETTING_KEYS.tool_set_name, String(tool_set_name));
    });

    $effect(() => {
        apply_theme_mode(theme_mode);
        void save_ui_setting(UI_SETTING_KEYS.theme_mode, String(theme_mode));
    });

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

    function close_save_prompt(value) {
        if (save_prompt_resolve) {
            save_prompt_resolve(value);
        }
        save_prompt_resolve = null;
        save_prompt_open = false;
        save_prompt_title = "";
        save_prompt_value = "";
    }

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

    function handle_global_shortcuts(event) {
        const key = event.key.toLowerCase();

        if (event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey) {
            if (key == "k") {
                event.preventDefault();
                kerning_editor_ref?.focusEditor();
                return;
            }
            if (key == "c") {
                event.preventDefault();
                config_editor_ref?.focusEditor();
                return;
            }
            if (key == "u") {
                event.preventDefault();
                glyph_data_editor_1?.focusEditor();
                return;
            }
            if (key == "g") {
                event.preventDefault();
                glyph_data_editor_2?.focusEditor();
                return;
            }
            if (key == "o") {
                event.preventDefault();
                glyph_data_editor_1?.focusGlyphSelector();
                return;
            }
            if (key == "e") {
                event.preventDefault();
                glyph_data_editor_2?.focusGlyphSelector();
                return;
            }
        }

        if (!event.metaKey || event.ctrlKey || event.altKey) {
            return;
        }

        if (event.shiftKey && key == "c") {
            event.preventDefault();
            void compile(event);
            return;
        }

        if (event.key == "@" || (event.shiftKey && event.code == "Digit2")) {
            event.preventDefault();
            void save_content_same_name(event);
            return;
        }

        if (event.key == "*" || (event.shiftKey && event.code == "Digit8")) {
            event.preventDefault();
            void save_tool_set_same_name(event);
            return;
        }

        if (event.key == "/" || event.key == "?") {
            event.preventDefault();
            void save_font(event);
        }
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
                void get_content_names(null);
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

        await load_persisted_ui_state();
        apply_theme_mode(theme_mode);

        await get_tool_set_names(null);
        if (tool_set_name != "" && tool_set_names.includes(tool_set_name)) {
            await get_tool_set_data(null);
        }
        await get_glyph_set_names(null);
        await get_font(null);
        await get_content_names(null);
        await get_content(null);
        await get_font_names(null);
        should_persist_ui_state = true;

        const unlisten_msg = await listen("msg", async (event) => {
            if (event.payload == "compile_ended") {
                fontname = "generated";
                await get_font(null);
                await get_font_names(event);
                ready_to_compile = true;
            }
        });

        const unlisten_error = await listen("error", async (event) => {
            error_msg = format_error_message(event.payload);
            open_alert_dialog("Compile Error", error_msg);
            ready_to_compile = true;
        });

        window.addEventListener("keydown", handle_global_shortcuts);

        return () => {
            window.removeEventListener("keydown", handle_global_shortcuts);
            unlisten_msg();
            unlisten_error();
        };
    });

    async function get_tool_set_names(event) {
        tool_set_names = await invoke("get_tool_set_names", {});
    }

    async function get_tool_set_data(event) {
        try {
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
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
        }
    }

    async function save_tool_set_as(event) {
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
        try {
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
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
        }
    }

    async function save_tool_set_same_name(event) {
        if (tool_set_name.trim() == "") {
            return;
        }
        try {
            await glyph_data_editor_1.save(event);
            await glyph_data_editor_2.save(event);
            await config_editor_ref.save(event);
            await kerning_editor_ref.save(event);
            await invoke("save_tool_set", {
                toolSet: {
                    config_name,
                    kerning_name,
                    glyph_set,
                },
                toolSetName: tool_set_name,
            });
            await get_tool_set_names(null);
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
        }
    }

    async function delete_tool_set(event) {
        if (tool_set_name == "") {
            return;
        }
        await invoke("delete_tool_set", {
            toolSetName: tool_set_name,
        });
        await get_tool_set_names(null);
    }

    async function get_glyph_set_names(event) {
        glyph_set_names = await invoke("get_glyph_set_names", {});
    }

    async function change_glyph_set(event) {
        await glyph_data_editor_1.save(event);
        await glyph_data_editor_2.save(event);
        await glyph_data_editor_1.loadGlyphDataWithoutSave(event);
        await glyph_data_editor_2.loadGlyphDataWithoutSave(event);
    }

    async function delete_glyph_set(event) {
        if (glyph_set == "") {
            return;
        }
        await invoke("delete_glyph_set", {
            glyphSet: glyph_set,
        });
        await get_glyph_set_names(null);
    }

    async function get_font_names(event) {
        fontnames = await invoke("get_font_names", {});
    }

    async function get_font(event) {
        let data = await invoke("get_font_data", {
            fontName: fontname,
        });
        let new_font = null;

        for (let face of document.fonts) {
            if (face.family.replaceAll('"', "") == "Linear Korean") {
                document.fonts.delete(face);
            }
        }
        if (data.length > 0) {
            new_font = new FontFace("Linear Korean", new Uint8Array(data));
            await new_font.load();
        }
        if (new_font) {
            document.fonts.add(new_font);
        }
    }

    async function delete_font(event) {
        await invoke("delete_font", {
            fontName: fontname,
        });
        await get_font_names(null);
    }

    async function save_font(event) {
        if (fontname == "") {
            return;
        }
        let next_font_name = await request_save_name("Save font as", fontname || "");
        if (next_font_name == null) {
            return;
        }
        invoke("save_font", {
            oldName: fontname,
            newName: next_font_name,
        }).catch(async function (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
        });
        await get_font_names(null);
        fontname = next_font_name;
    }

    async function get_content_names(event) {
        content_names = await invoke("get_content_names", {});
    }

    async function get_content(event) {
        content = await invoke("get_content", {
            contentName: content_name,
        });
    }

    async function save_content_as(event) {
        let next_content_name = await request_save_name(
            "Save content as filename",
            content_name || "",
        );
        if (next_content_name == null) {
            return;
        }
        await invoke("save_content", {
            content,
            contentName: next_content_name,
        });
        content_name = next_content_name;
        await get_content_names(null);
    }

    async function save_content_same_name(event) {
        if (content_name.trim() == "") {
            return;
        }
        await invoke("save_content", {
            content,
            contentName: content_name,
        });
        await get_content_names(null);
    }

    async function compile(event) {
        if (glyph_set == "" || config_name == "" || kerning_name == "") {
            return;
        }
        ready_to_compile = false;
        try {
            await kerning_editor_ref.save(event);
            await config_editor_ref.save(event);
            await glyph_data_editor_1.save(event);
            await glyph_data_editor_2.save(event);
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

<main class="flex h-screen min-h-0 flex-col bg-[hsl(var(--background))] text-[hsl(var(--foreground))]">
    <header class="border-b border-[hsl(var(--border))] bg-[hsl(var(--card))/0.88] backdrop-blur">
        <div class="flex flex-wrap items-end gap-1 p-1">
            <div class="space-y-1">
                <div class="ui-label">Content</div>
                <div class="flex items-center gap-2">
                    <select class="ui-select min-w-[11rem]" bind:value={content_name} onchange={get_content}>
                        {#each content_names as item}
                            <option value={item}>{item}</option>
                        {/each}
                    </select>
                    <button class="ui-button-secondary" onclick={get_content_names} title="Refresh content list" aria-label="Refresh content list">
                        <RefreshCcw class="h-4 w-4" />
                    </button>
                    <button class="ui-button-secondary" onclick={get_content} title="Load content" aria-label="Load content">
                        <FolderOpen class="h-4 w-4" />
                    </button>
                    <button class="ui-button-secondary" onclick={save_content_same_name} title="Save content" aria-label="Save content">
                        <Save class="h-4 w-4" />
                    </button>
                    <button class="ui-button-ghost" onclick={save_content_as} title="Save content as" aria-label="Save content as">Save As</button>
                </div>
            </div>

            <div class="space-y-1">
                <div class="ui-label">Preview</div>
                <div class="flex items-center gap-2">
                    <label class="ui-label sr-only" for="char-size">Size</label>
                    <select id="char-size" class="ui-select" bind:value={char_size}>
                        <option value={16}>16</option>
                        <option value={24}>24</option>
                        <option value={48}>48</option>
                        <option value={64}>64</option>
                        <option value={100}>100</option>
                    </select>
                    <input class="ui-input w-14 text-xl text-center border-0" type="text" bind:value={char} maxlength="1" />
                    <span class="linkor text-xl">{char}</span>
                </div>
            </div>

            <div class="space-y-1">
                <div class="ui-label">Font</div>
                <div class="flex items-center gap-2">
                    <select class="ui-select min-w-[10rem]" bind:value={fontname} onchange={get_font}>
                        {#each fontnames as item}
                            <option value={item}>{item}</option>
                        {/each}
                    </select>
                    <button class="ui-button-secondary" onclick={get_font_names} title="Refresh fonts" aria-label="Refresh fonts">
                        <RefreshCcw class="h-4 w-4" />
                    </button>
                    <button class="ui-button-secondary" onclick={delete_font} title="Delete font" aria-label="Delete font">
                        <Trash2 class="h-4 w-4" />
                    </button>
                    <button class="ui-button-ghost" onclick={save_font} title="Save font as" aria-label="Save font as">Save As</button>
                </div>
            </div>

            <div class="space-y-1">
                <div class="ui-label">Toolset</div>
                <div class="flex items-center gap-2">
                    <select class="ui-select min-w-[10rem]" bind:value={tool_set_name} onchange={get_tool_set_data}>
                        {#each tool_set_names as item}
                            <option value={item}>{item}</option>
                        {/each}
                    </select>
                    <button class="ui-button-secondary" onclick={save_tool_set_same_name} title="Save tool set" aria-label="Save tool set">
                        <Save class="h-4 w-4" />
                    </button>
                    <button class="ui-button-secondary" onclick={delete_tool_set} title="Delete tool set" aria-label="Delete tool set">
                        <Trash2 class="h-4 w-4" />
                    </button>
                    <button class="ui-button-ghost" onclick={save_tool_set_as} title="Save tool set as" aria-label="Save tool set as">Save As</button>
                </div>
            </div>

            <div class="space-y-1">
                <div class="ui-label">Glyph Set</div>
                <div class="flex items-center gap-2">
                    <select class="ui-select min-w-[10rem]" bind:value={glyph_set} onchange={change_glyph_set}>
                        {#each glyph_set_names as item}
                            <option value={item}>{item}</option>
                        {/each}
                    </select>
                    <button class="ui-button-secondary" onclick={get_glyph_set_names} title="Refresh glyph sets" aria-label="Refresh glyph sets">
                        <RefreshCcw class="h-4 w-4" />
                    </button>
                    <button class="ui-button-secondary" onclick={delete_glyph_set} title="Delete glyph set" aria-label="Delete glyph set">
                        <Trash2 class="h-4 w-4" />
                    </button>
                </div>
            </div>

            <div class="space-y-1">
                <div class="ui-label"></div>
                <div class="flex items-center gap-2">
                <button class="ui-button-secondary" onclick={toggle_theme_mode} title="Toggle theme" aria-label="Toggle theme">
                    {#if theme_mode == "dark"}
                        <Sun class="h-4 w-4" />
                    {:else}
                        <Moon class="h-4 w-4" />
                    {/if}
                </button>
                <button class="ui-button-primary flex items-center justify-center" onclick={compile} disabled={!ready_to_compile}>
                    <Hammer class="h-4 w-4" />
                </button>
                </div>
            </div>
        </div>
        <!--
        <div class="flex flex-wrap items-center gap-2 border-t border-[hsl(var(--border))] px-4 py-2 text-xs text-[hsl(var(--muted-foreground))]">
            <span>Shortcuts:</span>
            <span class="ui-kbd">Ctrl+K</span><span>Kerning</span>
            <span class="ui-kbd">Ctrl+C</span><span>Config</span>
            <span class="ui-kbd">Ctrl+G</span><span>Glyph 1</span>
            <span class="ui-kbd">Ctrl+D</span><span>Glyph 2</span>
            <span class="ui-kbd">Ctrl+U</span><span>Glyph 1 Selector</span>
            <span class="ui-kbd">Ctrl+G</span><span>Glyph 2 Selector</span>
            <span class="ui-kbd">Cmd+Shift+C</span><span>Compile</span>
            <span class="ui-kbd">Cmd+@</span><span>Save Content</span>
            <span class="ui-kbd">Cmd+/</span><span>Save Font As</span>
            <span class="ui-kbd">Cmd+*</span><span>Save Toolset</span>
        </div>
        -->
    </header>

    <section class="grid min-h-0 flex-1 grid-cols-1 gap-4 p-4 xl:grid-cols-[minmax(0,1.45fr)_minmax(0,1fr)]">
        <div class="ui-card flex min-h-0 flex-col p-4">
            <div class="mb-2 flex items-center justify-between">
                <div class="ui-label">Content Preview</div>
                <div class="text-xs text-[hsl(var(--muted-foreground))]">Top: generated font, Bottom: source text</div>
            </div>
            <div class="flex min-h-0 flex-1 flex-col gap-3">
                <textarea
                    class="h-full min-h-0 flex-1 resize-none rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--background))] p-3 linkor"
                    bind:value={content}
                    style:font-size={char_size + "px"}
                ></textarea>
                <textarea
                    class="h-44 resize-none rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--background))] p-3"
                    bind:value={content}
                ></textarea>
            </div>
        </div>

        <div class="flex min-h-0 flex-col gap-4">
            <div class="grid h-64 grid-cols-1 gap-4 lg:grid-cols-2">
                <KerningEditor
                    bind:this={kerning_editor_ref}
                    bind:kerning_name
                    {theme_mode}
                ></KerningEditor>
                <ConfigEditor
                    bind:this={config_editor_ref}
                    bind:config_name
                    {theme_mode}
                ></ConfigEditor>
            </div>

            <div class="grid grow min-h-0 grid-cols-2 gap-4">
                <GlyphDataEditor
                    bind:this={glyph_data_editor_1}
                    {glyph_set}
                    editor_label="Glyph Editor 1"
                    {theme_mode}
                ></GlyphDataEditor>
                <GlyphDataEditor
                    bind:this={glyph_data_editor_2}
                    {glyph_set}
                    editor_label="Glyph Editor 2"
                    {theme_mode}
                ></GlyphDataEditor>
            </div>
        </div>
    </section>

    {#if save_prompt_open}
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
            <div class="ui-card w-full max-w-md p-4">
                <div class="text-sm font-semibold">{save_prompt_title}</div>
                <input
                    class="ui-input mt-3 w-full"
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
                    <button class="ui-button-ghost" onclick={() => close_save_prompt(null)}>Cancel</button>
                    <button class="ui-button-primary" onclick={confirm_save_prompt}>Save</button>
                </div>
            </div>
        </div>
    {/if}

    {#if alert_dialog_open}
        <div class="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4">
            <div class="ui-card w-full max-w-2xl p-4">
                <div class="text-sm font-semibold">{alert_dialog_title}</div>
                <pre class="mt-3 max-h-80 overflow-auto whitespace-pre-wrap rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--muted))] p-2 text-sm">{alert_dialog_message}</pre>
                <div class="mt-3 flex justify-end">
                    <button class="ui-button-primary" onclick={close_alert_dialog}>OK</button>
                </div>
            </div>
        </div>
    {/if}
</main>

<style>
    @font-face {
        font-family: "Linear Korean";
        src: url("");
    }

    .linkor {
        font-family: "Linear Korean", "Noto Sans KR", sans-serif;
    }
</style>
