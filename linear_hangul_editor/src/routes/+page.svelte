<script>
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onMount, tick } from "svelte";
    import { Menu, Submenu, PredefinedMenuItem } from "@tauri-apps/api/menu";
    import JSON5 from "json5";
    import {
        BookmarkPlus,
        CopyPlus,
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
    import EvolutionDialog from "$lib/evolution_dialog.svelte";

    const EVOLUTION_MUTATION_COUNT = 8;
    const EVOLUTION_SYSTEM_RENDERING_INDEX = 1;
    const EVOLUTION_RENDERING_COUNT = EVOLUTION_MUTATION_COUNT + 2;

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
    let collision_check_enabled = $state(true);

    let save_prompt_open = $state(false);
    let save_prompt_title = $state("");
    let save_prompt_value = $state("");
    let save_prompt_resolve = null;
    let save_prompt_input_ref = $state(null);

    let alert_dialog_open = $state(false);
    let alert_dialog_title = $state("Error");
    let alert_dialog_message = $state("");
    let collision_debug_payload = $state(null);
    let collision_debug_image_url = $state("");
    let evolution_dialog_open = $state(false);
    let evolution_config_names = $state([]);
    let evolution_config_name = $state("");
    let evolution_config_data = $state("");
    let evolution_config_drafts = $state({});
    let evolution_generation = $state(0);
    let evolution_base_rendering = $state(0);
    let evolution_selected_rendering = $state(0);
    let evolution_renderings = $state(build_evolution_renderings("", 0));
    let evolution_selected_config_data = $state("");
    let evolution_selected_kerning_data = $state("");
    let evolution_session_id = $state(null);
    let evolution_font_families = $state([]);
    let evolution_renderings_from_backend = $state(false);
    let evolution_state_initialized = $state(false);

    let theme_mode = $state("light");
    let should_persist_ui_state = false;

    const UI_SETTING_KEYS = {
        char_size: "char_size",
        content_name: "content_name",
        char: "char",
        fontname: "fontname",
        tool_set_name: "tool_set_name",
        theme_mode: "theme_mode",
        collision_check_enabled: "collision_check_enabled",
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
        const collision_check_value = await get_setting_or_default(
            UI_SETTING_KEYS.collision_check_enabled,
            String(collision_check_enabled),
        );
        collision_check_enabled = collision_check_value != "false";
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

    $effect(() => {
        void save_ui_setting(
            UI_SETTING_KEYS.collision_check_enabled,
            String(collision_check_enabled),
        );
    });

    $effect(() => {
        if (!evolution_dialog_open || evolution_renderings_from_backend) {
            return;
        }
        const seed_config_data = evolution_selected_config_data ?? "";
        const seed_kerning_data = evolution_selected_kerning_data ?? "";
        evolution_renderings = build_evolution_renderings(
            get_evolution_preview_text(),
            evolution_generation,
            seed_config_data,
            seed_kerning_data,
        );
    });

    async function refresh_all_lists(event) {
        await Promise.all([
            get_content_names(event),
            get_font_names(event),
            get_tool_set_names(event),
            get_glyph_set_names(event),
        ]);
    }

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

    function handle_config_editor_error(title, message_text) {
        error_msg = format_error_message(message_text);
        open_alert_dialog(title, error_msg);
    }

    function clear_collision_debug() {
        collision_debug_payload = null;
        collision_debug_image_url = "";
    }

    function build_collision_debug_image_url(payload) {
        if (typeof document == "undefined") {
            return "";
        }
        const width = Number(payload?.width ?? 0);
        const height = Number(payload?.height ?? 0);
        if (!Number.isFinite(width) || !Number.isFinite(height)) {
            return "";
        }
        if (width <= 0 || height <= 0) {
            return "";
        }
        const canvas = document.createElement("canvas");
        canvas.width = width;
        canvas.height = height;
        const context = canvas.getContext("2d");
        if (context == null) {
            return "";
        }
        const image_data = context.createImageData(width, height);
        const data = image_data.data;

        function paint_pixels(points, r, g, b, a) {
            for (const point of points ?? []) {
                const x = Number(point?.[0]);
                const y = Number(point?.[1]);
                if (!Number.isFinite(x) || !Number.isFinite(y)) {
                    continue;
                }
                if (x < 0 || y < 0 || x >= width || y >= height) {
                    continue;
                }
                const idx = (Math.floor(y) * width + Math.floor(x)) * 4;
                data[idx] = r;
                data[idx + 1] = g;
                data[idx + 2] = b;
                data[idx + 3] = a;
            }
        }

        paint_pixels(payload?.component_a, 120, 120, 120, 180);
        paint_pixels(payload?.component_b, 54, 133, 255, 180);
        paint_pixels(payload?.overlap, 220, 38, 38, 255);
        context.putImageData(image_data, 0, 0);
        return canvas.toDataURL("image/png");
    }

    function format_config_json_for_editor(value) {
        let out = JSON.stringify(value, null, 2);
        if (!out.endsWith("\n")) {
            out += "\n";
        }
        return out;
    }

    function merge_config_text_for_main_editor(variant_config_text) {
        const selected_text = String(variant_config_text ?? "");
        const current_text = String(config_editor_ref?.getConfigData?.() ?? "");
        try {
            const selected_obj = JSON5.parse(selected_text);
            if (
                selected_obj == null ||
                Array.isArray(selected_obj) ||
                typeof selected_obj != "object"
            ) {
                return selected_text;
            }
            let current_obj = {};
            if (current_text.trim() != "") {
                const parsed_current = JSON5.parse(current_text);
                if (
                    parsed_current != null &&
                    !Array.isArray(parsed_current) &&
                    typeof parsed_current == "object"
                ) {
                    current_obj = parsed_current;
                }
            }
            const merged = { ...current_obj, ...selected_obj };
            return format_config_json_for_editor(merged);
        } catch (_) {
            return selected_text;
        }
    }

    function read_space_debug_from_config_text(config_text) {
        const source = String(config_text ?? "");
        try {
            const parsed = JSON5.parse(source);
            if (parsed == null || Array.isArray(parsed) || typeof parsed != "object") {
                return { space_width: null, space_width_ratio: null };
            }
            return {
                space_width:
                    Object.prototype.hasOwnProperty.call(parsed, "space_width")
                        ? parsed.space_width
                        : null,
                space_width_ratio:
                    Object.prototype.hasOwnProperty.call(parsed, "space_width_ratio")
                        ? parsed.space_width_ratio
                        : null,
            };
        } catch (error) {
            console.debug("[evolution][render-ui] failed to parse config text", error);
            return { space_width: null, space_width_ratio: null };
        }
    }

    function set_collision_debug(payload) {
        collision_debug_payload = payload;
        collision_debug_image_url = build_collision_debug_image_url(payload);
    }

    function close_alert_dialog() {
        alert_dialog_open = false;
        clear_collision_debug();
    }

    function get_evolution_preview_text() {
        return (content || "").slice(0, 200);
    }

    function is_system_evolution_rendering(index) {
        return index == EVOLUTION_SYSTEM_RENDERING_INDEX;
    }

    function evolution_backend_index_from_ui(index) {
        if (typeof index != "number" || !Number.isFinite(index)) {
            return null;
        }
        if (index < 0) {
            return null;
        }
        if (is_system_evolution_rendering(index)) {
            return null;
        }
        return index > EVOLUTION_SYSTEM_RENDERING_INDEX ? index - 1 : index;
    }

    function build_system_hangul_rendering(
        text,
        config_data = "",
        kerning_data = "",
        render_version = 0,
    ) {
        return {
            label: "Standard System Hangul",
            text,
            font_family: "system-ui",
            config_data,
            kerning_data,
            render_version,
            is_system_font: true,
        };
    }

    function build_evolution_renderings(
        preview_text,
        generation,
        seed_config_data = "",
        seed_kerning_data = "",
    ) {
        const text =
            preview_text.length > 0
                ? preview_text
                : "No content is loaded. Select a content file to preview evolution renderings.";
        const renderings = [
            {
                label: "Base Variant",
                text,
                font_family: "Linear Korean",
                config_data: seed_config_data,
                kerning_data: seed_kerning_data,
                render_version: 0,
                is_system_font: false,
            },
            build_system_hangul_rendering(
                text,
                seed_config_data,
                seed_kerning_data,
                generation,
            ),
        ];
        for (let idx = 1; idx <= EVOLUTION_MUTATION_COUNT; idx += 1) {
            renderings.push({
                label: `Mutation ${idx} · G${generation}`,
                text,
                font_family: "Linear Korean",
                config_data: seed_config_data,
                kerning_data: seed_kerning_data,
                render_version: 0,
                is_system_font: false,
            });
        }
        return renderings;
    }

    function sync_selected_variant_editors(index = evolution_selected_rendering) {
        const selected = evolution_renderings[index];
        const next_config_data = selected?.config_data ?? "";
        const next_kerning_data = selected?.kerning_data ?? "";
        if (evolution_selected_config_data != next_config_data) {
            evolution_selected_config_data = next_config_data;
        }
        if (evolution_selected_kerning_data != next_kerning_data) {
            evolution_selected_kerning_data = next_kerning_data;
        }
    }

    function clear_evolution_preview_fonts() {
        if (typeof document == "undefined") {
            evolution_font_families = [];
            return;
        }
        const known_families = new Set(evolution_font_families);
        for (const face of document.fonts) {
            const family = face.family.replaceAll('"', "");
            if (!known_families.has(family)) {
                continue;
            }
            document.fonts.delete(face);
        }
        evolution_font_families = [];
    }

    function get_evolution_preview_family(index) {
        return `LinearKoreanEvolutionSlot${index + 1}`;
    }

    function remove_font_faces_by_family(family) {
        if (typeof document == "undefined") {
            return;
        }
        for (const face of document.fonts) {
            if (face.family.replaceAll('"', "") == family) {
                document.fonts.delete(face);
            }
        }
    }

    async function build_evolution_renderings_from_result(
        preview_text,
        generation,
        evolve_result,
    ) {
        const text =
            preview_text.length > 0
                ? preview_text
                : "No content is loaded. Select a content file to preview evolution renderings.";
        const items = Array.isArray(evolve_result?.items) ? evolve_result.items : [];
        if (items.length != EVOLUTION_MUTATION_COUNT + 1) {
            throw new Error(
                `Invalid evolve response: expected ${EVOLUTION_MUTATION_COUNT + 1} preview fonts, got ${items.length}.`,
            );
        }
        const original_item = items[0];
        const original_config_data = String(original_item?.configData ?? "");
        const original_kerning_data = String(original_item?.kerningData ?? "");
        if (typeof document == "undefined") {
            const renderings = [
                {
                    label:
                        typeof original_item?.label == "string" &&
                        original_item.label.length > 0
                            ? original_item.label
                            : "Base Variant",
                    text,
                    font_family: "Linear Korean",
                    config_data: original_config_data,
                    kerning_data: original_kerning_data,
                    render_version: generation,
                    is_system_font: false,
                },
                build_system_hangul_rendering(
                    text,
                    original_config_data,
                    original_kerning_data,
                    generation,
                ),
            ];
            for (let idx = 1; idx < items.length; idx += 1) {
                const item = items[idx];
                renderings.push({
                    label:
                        typeof item?.label == "string" && item.label.length > 0
                            ? item.label
                            : `Mutation ${idx} · G${generation}`,
                    text,
                    font_family: "Linear Korean",
                    config_data: String(item?.configData ?? ""),
                    kerning_data: String(item?.kerningData ?? ""),
                    render_version: generation,
                    is_system_font: false,
                });
            }
            return renderings;
        }

        clear_evolution_preview_fonts();
        const loaded_families = [];
        const renderings = Array.from({ length: EVOLUTION_RENDERING_COUNT });
        for (let idx = 0; idx < items.length; idx += 1) {
            const item = items[idx];
            const font_name = String(item?.fontName ?? "").trim();
            if (font_name == "") {
                throw new Error(`Invalid evolve response item at index ${idx}: missing font_name.`);
            }
            const font_data = await invoke("get_font_data", {
                fontName: font_name,
            });
            if (!Array.isArray(font_data) || font_data.length == 0) {
                throw new Error(`Generated evolution font '${font_name}' is empty.`);
            }
            const ui_index = idx > 0 ? idx + 1 : idx;
            const family = get_evolution_preview_family(ui_index);
            remove_font_faces_by_family(family);
            const font_face = new FontFace(family, new Uint8Array(font_data));
            await font_face.load();
            document.fonts.add(font_face);
            loaded_families.push(family);
            renderings[ui_index] = {
                label:
                    typeof item?.label == "string" && item.label.length > 0
                        ? item.label
                        : idx == 0
                          ? "Base Variant"
                          : `Mutation ${idx} · G${generation}`,
                text,
                font_family: family,
                config_data: String(item?.configData ?? ""),
                kerning_data: String(item?.kerningData ?? ""),
                render_version: generation,
                is_system_font: false,
            };
        }
        renderings[EVOLUTION_SYSTEM_RENDERING_INDEX] = build_system_hangul_rendering(
            text,
            original_config_data,
            original_kerning_data,
            generation,
        );
        evolution_font_families = loaded_families;
        return renderings;
    }

    async function apply_evolution_rendering_item(index, item) {
        if (is_system_evolution_rendering(index)) {
            throw new Error("System Hangul rendering cannot be replaced.");
        }
        const font_name = String(item?.fontName ?? "").trim();
        if (font_name == "") {
            throw new Error("Invalid evolve response item: missing font_name.");
        }
        const next_config_data = String(item?.configData ?? "");
        const next_kerning_data = String(item?.kerningData ?? "");
        const next_label = String(item?.label ?? evolution_renderings[index]?.label ?? "");

        let font_family = evolution_renderings[index]?.font_family ?? "Linear Korean";
        if (typeof document != "undefined") {
            const font_data = await invoke("get_font_data", {
                fontName: font_name,
            });
            if (!Array.isArray(font_data) || font_data.length == 0) {
                throw new Error(`Generated evolution font '${font_name}' is empty.`);
            }
            const family = get_evolution_preview_family(index);
            remove_font_faces_by_family(family);
            const font_face = new FontFace(family, new Uint8Array(font_data));
            await font_face.load();
            document.fonts.add(font_face);
            evolution_font_families = Array.from(new Set([...evolution_font_families, family]));
            font_family = family;
        }

        evolution_renderings = evolution_renderings.map((rendering, idx) => {
            if (idx != index) {
                return rendering;
            }
            return {
                ...rendering,
                label: next_label,
                font_family,
                config_data: next_config_data,
                kerning_data: next_kerning_data,
                render_version: Number(rendering?.render_version ?? 0) + 1,
            };
        });
        if (index == evolution_selected_rendering) {
            sync_selected_variant_editors(index);
        }
        evolution_renderings_from_backend = true;
    }

    function cache_selected_variant_draft(kind, event) {
        if (
            evolution_selected_rendering < 0 ||
            evolution_selected_rendering >= evolution_renderings.length
        ) {
            return;
        }
        const next_data =
            event?.currentTarget?.value != null
                ? event.currentTarget.value
                : kind == "config"
                  ? evolution_selected_config_data
                  : evolution_selected_kerning_data;
        if (kind == "config") {
            evolution_selected_config_data = next_data;
        } else {
            evolution_selected_kerning_data = next_data;
        }
        evolution_renderings = evolution_renderings.map((rendering, idx) => {
            if (idx != evolution_selected_rendering) {
                return rendering;
            }
            return {
                ...rendering,
                config_data:
                    kind == "config" ? next_data : rendering.config_data ?? "",
                kerning_data:
                    kind == "kerning" ? next_data : rendering.kerning_data ?? "",
            };
        });
    }

    async function get_evolution_config_names() {
        evolution_config_names = await invoke("get_evolution_config_names", {});
        if (evolution_config_names.length == 0) {
            evolution_config_name = "";
            return;
        }
        if (
            evolution_config_name == "" ||
            !evolution_config_names.includes(evolution_config_name)
        ) {
            evolution_config_name = evolution_config_names[0];
        }
    }

    async function get_evolution_config_data(force_reload = false) {
        const should_force_reload = force_reload === true;
        if (evolution_config_name.trim() == "") {
            return;
        }
        const cached_draft = evolution_config_drafts[evolution_config_name];
        if (!should_force_reload && cached_draft !== undefined) {
            evolution_config_data = cached_draft;
            return;
        }
        const loaded = await invoke("get_evolution_config_data", {
            evolutionName: evolution_config_name,
        });
        evolution_config_data = loaded;
        evolution_config_drafts = {
            ...evolution_config_drafts,
            [evolution_config_name]: loaded,
        };
    }

    function cache_current_evolution_draft(event) {
        if (evolution_config_name.trim() == "") {
            return;
        }
        const next_data =
            event?.currentTarget?.value != null
                ? event.currentTarget.value
                : evolution_config_data;
        evolution_config_data = next_data;
        evolution_config_drafts = {
            ...evolution_config_drafts,
            [evolution_config_name]: next_data,
        };
    }

    async function save_evolution_config_same_name() {
        if (evolution_config_name.trim() == "") {
            return;
        }
        try {
            await invoke("save_evolution_config", {
                evolutionData: evolution_config_data,
                evolutionName: evolution_config_name,
            });
            await get_evolution_config_names();
            await get_evolution_config_data(true);
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
        }
    }

    async function save_evolution_config_as() {
        let next_evolution_name = await request_save_name(
            "Save evolution config as",
            evolution_config_name || "default",
        );
        if (next_evolution_name == null) {
            return;
        }
        next_evolution_name = next_evolution_name.trim();
        if (next_evolution_name == "") {
            return;
        }
        try {
            await invoke("save_evolution_config", {
                evolutionData: evolution_config_data,
                evolutionName: next_evolution_name,
            });
            evolution_config_name = next_evolution_name;
            await get_evolution_config_names();
            await get_evolution_config_data(true);
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
        }
    }

    async function rename_evolution_config() {
        const previous_name = evolution_config_name.trim();
        if (previous_name == "") {
            return;
        }
        let next_evolution_name = await request_save_name(
            "Rename evolution config",
            previous_name,
        );
        if (next_evolution_name == null) {
            return;
        }
        next_evolution_name = next_evolution_name.trim();
        if (
            next_evolution_name == "" ||
            next_evolution_name == previous_name
        ) {
            return;
        }
        try {
            await invoke("rename_evolution_config", {
                oldName: previous_name,
                newName: next_evolution_name,
            });
            const renamed_draft =
                evolution_config_drafts[previous_name] ?? evolution_config_data;
            const { [previous_name]: _discarded, ...remaining_drafts } =
                evolution_config_drafts;
            evolution_config_drafts = {
                ...remaining_drafts,
                [next_evolution_name]: renamed_draft,
            };
            evolution_config_name = next_evolution_name;
            await get_evolution_config_names();
            await get_evolution_config_data(true);
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
        }
    }

    async function delete_evolution_config() {
        const target_name = evolution_config_name.trim();
        if (target_name == "") {
            return;
        }
        try {
            await invoke("delete_evolution_config", {
                evolutionName: target_name,
            });
            const { [target_name]: _discarded, ...remaining_drafts } =
                evolution_config_drafts;
            evolution_config_drafts = remaining_drafts;
            await get_evolution_config_names();
            if (evolution_config_name.trim() == "") {
                evolution_config_data = "";
                return;
            }
            await get_evolution_config_data(true);
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
        }
    }

    async function open_evolution_dialog() {
        try {
            await get_evolution_config_names();
            if (!evolution_state_initialized) {
                await get_evolution_config_data();
                clear_evolution_preview_fonts();
                evolution_session_id = null;
                evolution_renderings_from_backend = false;
                evolution_generation = 0;
                evolution_base_rendering = 0;
                evolution_selected_rendering = 0;
                const seed_config_data = await invoke("get_config_data", {
                    configName: config_name,
                });
                const seed_kerning_data = await invoke("get_kerning_data", {
                    kerningName: kerning_name,
                });
                evolution_renderings = build_evolution_renderings(
                    get_evolution_preview_text(),
                    evolution_generation,
                    seed_config_data,
                    seed_kerning_data,
                );
                sync_selected_variant_editors(0);
                evolution_state_initialized = true;
            } else {
                sync_selected_variant_editors(evolution_selected_rendering);
            }
            evolution_dialog_open = true;
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
        }
    }

    function close_evolution_dialog() {
        cache_current_evolution_draft();
        evolution_dialog_open = false;
    }

    async function evolve_shell(
        seed_index = evolution_base_rendering,
        reset_to_root = false,
        seed_override = null,
    ) {
        if (
            !ready_to_compile ||
            evolution_config_name.trim() == "" ||
            glyph_set.trim() == "" ||
            config_name.trim() == "" ||
            kerning_name.trim() == ""
        ) {
            return;
        }

        clear_collision_debug();
        ready_to_compile = false;
        const preview_text = get_evolution_preview_text();
        const next_generation = evolution_generation + 1;
        try {
            await invoke("save_evolution_config", {
                evolutionData: evolution_config_data,
                evolutionName: evolution_config_name,
            });
            await get_evolution_config_names();
            await get_evolution_config_data(true);
            await kerning_editor_ref.save(null);
            await config_editor_ref.save(null);
            await glyph_data_editor_1.save(null);
            await glyph_data_editor_2.save(null);

            const resolved_seed_index = is_system_evolution_rendering(seed_index)
                ? 0
                : seed_index;
            const seed_backend_index =
                evolution_backend_index_from_ui(resolved_seed_index);
            const seed_rendering = evolution_renderings[resolved_seed_index];
            const override_config_data = String(seed_override?.config_data ?? "");
            const override_kerning_data = String(seed_override?.kerning_data ?? "");
            const evolve_result = await invoke("evolve", {
                evolutionName: evolution_config_name,
                configName: config_name,
                kerningName: kerning_name,
                glyphSet: glyph_set,
                content: preview_text,
                checkCollision: collision_check_enabled,
                sessionId: evolution_session_id,
                seedIndex: seed_backend_index,
                resetToRoot: reset_to_root,
                seedConfigData:
                    !reset_to_root
                        ? (override_config_data != ""
                              ? override_config_data
                              : (seed_rendering?.config_data ??
                                evolution_selected_config_data))
                        : null,
                seedKerningData:
                    !reset_to_root
                        ? (override_kerning_data != ""
                              ? override_kerning_data
                              : (seed_rendering?.kerning_data ??
                                evolution_selected_kerning_data))
                        : null,
            });
            evolution_session_id = evolve_result?.sessionId ?? evolution_session_id;
            const applied_generation = Number(evolve_result?.generation ?? next_generation);
            const renderings = await build_evolution_renderings_from_result(
                preview_text,
                applied_generation,
                evolve_result,
            );
            evolution_generation = applied_generation;
            evolution_base_rendering = 0;
            evolution_selected_rendering = 0;
            evolution_renderings = renderings;
            evolution_renderings_from_backend = true;
            sync_selected_variant_editors(0);
        } catch (e) {
            clear_collision_debug();
            error_msg = format_error_message(e);
            open_alert_dialog("Evolution Error", error_msg);
        } finally {
            ready_to_compile = true;
        }
    }

    function reset_evolution_renderings() {
        evolution_base_rendering = 0;
        evolution_selected_rendering = 0;
        void evolve_shell(0, true);
    }

    function choose_evolution_rendering(index) {
        evolution_selected_rendering = index;
        if (!is_system_evolution_rendering(index)) {
            evolution_base_rendering = index;
        }
        sync_selected_variant_editors(index);
    }

    function evolve_from_rendering(index) {
        evolution_selected_rendering = index;
        if (is_system_evolution_rendering(index)) {
            sync_selected_variant_editors(index);
            return;
        }
        evolution_base_rendering = index;
        sync_selected_variant_editors(index);
        void evolve_shell(index, false);
    }

    async function render_again_from_rendering(index) {
        if (
            !ready_to_compile ||
            evolution_config_name.trim() == "" ||
            glyph_set.trim() == "" ||
            config_name.trim() == "" ||
            kerning_name.trim() == ""
        ) {
            return;
        }
        if (index < 0 || index >= evolution_renderings.length) {
            return;
        }
        if (is_system_evolution_rendering(index)) {
            return;
        }
        const target_backend_index = evolution_backend_index_from_ui(index);
        if (target_backend_index == null) {
            return;
        }
        const render_config_data = String(
            evolution_selected_config_data ??
                evolution_renderings[index]?.config_data ??
                "",
        );
        const render_kerning_data = String(
            evolution_selected_kerning_data ??
                evolution_renderings[index]?.kerning_data ??
                "",
        );
        if (render_config_data.trim() == "") {
            return;
        }

        clear_collision_debug();
        ready_to_compile = false;
        try {
            const space_debug = read_space_debug_from_config_text(render_config_data);
            console.debug("[evolution][render-ui] render request", {
                index,
                generation: evolution_generation,
                space_width: space_debug.space_width,
                space_width_ratio: space_debug.space_width_ratio,
            });
            await invoke("save_evolution_config", {
                evolutionData: evolution_config_data,
                evolutionName: evolution_config_name,
            });
            await get_evolution_config_names();
            await get_evolution_config_data(true);
            await glyph_data_editor_1.save(null);
            await glyph_data_editor_2.save(null);

            const rendered_item = await invoke("evolve_render_variant", {
                evolutionName: evolution_config_name,
                configName: config_name,
                kerningName: kerning_name,
                glyphSet: glyph_set,
                content: get_evolution_preview_text(),
                checkCollision: collision_check_enabled,
                renderConfigData: render_config_data,
                renderKerningData: render_kerning_data,
                targetIndex: target_backend_index,
                generation: evolution_generation,
                label: evolution_renderings[index]?.label ?? null,
            });
            const result_space_debug = read_space_debug_from_config_text(
                String(rendered_item?.configData ?? ""),
            );
            console.debug("[evolution][render-ui] render result", {
                index,
                font_name: rendered_item?.fontName ?? "",
                space_width: result_space_debug.space_width,
                space_width_ratio: result_space_debug.space_width_ratio,
            });

            await apply_evolution_rendering_item(index, rendered_item);
        } catch (e) {
            clear_collision_debug();
            error_msg = format_error_message(e);
            open_alert_dialog("Evolution Error", error_msg);
        } finally {
            ready_to_compile = true;
        }
    }

    function copy_selected_variant_to_main_editors() {
        const selected = evolution_renderings[evolution_selected_rendering];
        const next_config_data_raw = String(
            selected?.config_data ?? evolution_selected_config_data ?? "",
        );
        const next_config_data = merge_config_text_for_main_editor(next_config_data_raw);
        const next_kerning_data = String(
            selected?.kerning_data ?? evolution_selected_kerning_data ?? "",
        );
        config_editor_ref?.setConfigData?.(next_config_data);
        kerning_editor_ref?.setKerningData?.(next_kerning_data);
    }

    async function save_selected_variant_as(event) {
        if (
            !ready_to_compile ||
            glyph_set.trim() == "" ||
            evolution_selected_rendering < 0 ||
            evolution_selected_rendering >= evolution_renderings.length
        ) {
            return;
        }
        if (is_system_evolution_rendering(evolution_selected_rendering)) {
            return;
        }
        const selected = evolution_renderings[evolution_selected_rendering];
        const selected_config_data = String(
            evolution_selected_config_data ?? selected?.config_data ?? "",
        );
        const selected_kerning_data = String(
            evolution_selected_kerning_data ?? selected?.kerning_data ?? "",
        );
        if (selected_config_data.trim() == "") {
            return;
        }

        let next_name = await request_save_name(
            "Save evolution result as",
            evolution_config_name || tool_set_name || "",
        );
        if (next_name == null) {
            return;
        }
        next_name = next_name.trim();
        if (next_name == "") {
            return;
        }

        clear_collision_debug();
        ready_to_compile = false;
        try {
            await glyph_data_editor_1.save(event);
            await glyph_data_editor_2.save(event);
            await invoke("save_evolution_variant_as", {
                saveName: next_name,
                glyphSet: glyph_set,
                configData: selected_config_data,
                kerningData: selected_kerning_data,
                checkCollision: collision_check_enabled,
            });
            await get_font_names(null);
            await get_tool_set_names(null);
            await get_glyph_set_names(null);
            fontname = next_name;
            await get_font(null);
        } catch (e) {
            clear_collision_debug();
            error_msg = format_error_message(e);
            open_alert_dialog("Evolution Error", error_msg);
        } finally {
            ready_to_compile = true;
        }
    }

    async function replace_rendering_from_base(index) {
        if (
            !ready_to_compile ||
            evolution_config_name.trim() == "" ||
            glyph_set.trim() == "" ||
            config_name.trim() == "" ||
            kerning_name.trim() == ""
        ) {
            return;
        }
        if (index < 0 || index >= evolution_renderings.length) {
            return;
        }
        if (is_system_evolution_rendering(index)) {
            return;
        }
        const target_backend_index = evolution_backend_index_from_ui(index);
        if (target_backend_index == null) {
            return;
        }

        if (is_system_evolution_rendering(evolution_base_rendering)) {
            return;
        }
        const base_rendering = evolution_renderings[evolution_base_rendering];
        const base_config_data = String(
            base_rendering?.config_data ?? evolution_selected_config_data ?? "",
        );
        const base_kerning_data = String(
            base_rendering?.kerning_data ?? evolution_selected_kerning_data ?? "",
        );
        if (base_config_data.trim() == "") {
            return;
        }

        clear_collision_debug();
        ready_to_compile = false;
        try {
            await invoke("save_evolution_config", {
                evolutionData: evolution_config_data,
                evolutionName: evolution_config_name,
            });
            await get_evolution_config_names();
            await get_evolution_config_data(true);
            await glyph_data_editor_1.save(null);
            await glyph_data_editor_2.save(null);

            const replacement_item = await invoke("evolve_replace_variant", {
                evolutionName: evolution_config_name,
                configName: config_name,
                kerningName: kerning_name,
                glyphSet: glyph_set,
                content: get_evolution_preview_text(),
                checkCollision: collision_check_enabled,
                baseConfigData: base_config_data,
                baseKerningData: base_kerning_data,
                targetIndex: target_backend_index,
                generation: evolution_generation,
            });
            await apply_evolution_rendering_item(index, replacement_item);
        } catch (e) {
            clear_collision_debug();
            error_msg = format_error_message(e);
            open_alert_dialog("Evolution Error", error_msg);
        } finally {
            ready_to_compile = true;
        }
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
        void tick().then(() => {
            if (save_prompt_input_ref) {
                save_prompt_input_ref.focus();
                save_prompt_input_ref.select();
            }
        });
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

        if (event.metaKey && event.shiftKey) {
            if (key == "c") {
                event.preventDefault();
                void compile(event);
                return;
            }
            if (key == "p") {
                event.preventDefault();
                void compile_content(event);
                return;
            }
            if (key == "e" && evolution_dialog_open) {
                event.preventDefault();
                void evolve_shell();
                return;
            }
            if (key == "t") {
                event.preventDefault();
                collision_check_enabled = !collision_check_enabled;
                return;
            }
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
            text: "Refresh Lists",
            action: () => {
                void refresh_all_lists(null);
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
        await refresh_all_lists(null);
        await get_font(null);
        await get_content(null);
        should_persist_ui_state = true;

        const unlisten_msg = await listen("msg", async (event) => {
            if (event.payload == "compile_ended") {
                clear_collision_debug();
                const next_font_name = await finalize_compiled_font_name();
                if (next_font_name != "") {
                    fontname = next_font_name;
                }
                await get_font(null);
                await get_font_names(event);
                ready_to_compile = true;
            }
        });

        const unlisten_collision_debug = await listen(
            "collision_debug",
            async (event) => {
                set_collision_debug(event.payload);
            },
        );

        const unlisten_error = await listen("error", async (event) => {
            error_msg = format_error_message(event.payload);
            open_alert_dialog("Compile Error", error_msg);
            ready_to_compile = true;
        });

        window.addEventListener("keydown", handle_global_shortcuts);

        return () => {
            window.removeEventListener("keydown", handle_global_shortcuts);
            unlisten_msg();
            unlisten_collision_debug();
            unlisten_error();
        };
    });

    async function get_tool_set_names(event) {
        tool_set_names = await invoke("get_tool_set_names", {});
    }

    async function get_tool_set_data(event) {
        const warnings = [];
        const run_step = async (label, step) => {
            try {
                await step();
            } catch (e) {
                warnings.push(`${label}: ${format_error_message(e)}`);
            }
        };

        await run_step("Saving current config", async () => {
            await config_editor_ref.save(event);
        });
        await run_step("Saving current kerning", async () => {
            await kerning_editor_ref.save(event);
        });
        await run_step("Saving glyph editor 1", async () => {
            await glyph_data_editor_1.save(event);
        });
        await run_step("Saving glyph editor 2", async () => {
            await glyph_data_editor_2.save(event);
        });

        let data = null;
        try {
            data = await invoke("get_tool_set_data", {
                toolSetName: tool_set_name,
            });
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
            return;
        }

        glyph_set = data.glyph_set;
        config_name = data.config_name;
        kerning_name = data.kerning_name;

        await run_step(`Loading config '${config_name}'`, async () => {
            await config_editor_ref.load_config_data(event);
        });
        await run_step(`Loading kerning '${kerning_name}'`, async () => {
            await kerning_editor_ref.load_kerning_data(event);
        });
        await run_step(`Loading glyph set '${glyph_set}' in editor 1`, async () => {
            await glyph_data_editor_1.loadGlyphDataWithoutSave(event);
        });
        await run_step(`Loading glyph set '${glyph_set}' in editor 2`, async () => {
            await glyph_data_editor_2.loadGlyphDataWithoutSave(event);
        });
        await run_step(
            `Switching to font '${tool_set_name}' when available`,
            async () => {
                await switch_font_to_tool_set_if_present();
            },
        );

        if (warnings.length > 0) {
            error_msg = warnings.join("\n\n");
            open_alert_dialog("Toolset Loaded With Errors", error_msg);
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
        try {
            await invoke("save_font", {
                oldName: fontname,
                newName: next_font_name,
            });
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
            return;
        }
        await get_font_names(null);
        fontname = next_font_name;
        await get_font(null);
    }

    async function save_font_with_tool_set_name() {
        const source_font_name = String(fontname ?? "").trim();
        const target_font_name = String(tool_set_name ?? "").trim();
        if (source_font_name == "" || target_font_name == "") {
            return;
        }
        if (source_font_name == target_font_name) {
            return;
        }
        try {
            await invoke("save_font", {
                oldName: source_font_name,
                newName: target_font_name,
            });
            await get_font_names(null);
            fontname = target_font_name;
            await get_font(null);
        } catch (e) {
            error_msg = format_error_message(e);
            open_alert_dialog("Error", error_msg);
        }
    }

    async function switch_font_to_tool_set_if_present() {
        const matching_font_name = String(tool_set_name ?? "").trim();
        if (matching_font_name == "" || matching_font_name == fontname) {
            return;
        }
        await get_font_names(null);
        if (!fontnames.includes(matching_font_name)) {
            return;
        }
        fontname = matching_font_name;
        await get_font(null);
    }

    async function finalize_compiled_font_name() {
        await get_font_names(null);
        if (!fontnames.includes(DEFAULT_FONT_NAME)) {
            return fontname;
        }
        const next_font_name = String(tool_set_name ?? "").trim();
        if (
            next_font_name == "" ||
            next_font_name == DEFAULT_FONT_NAME ||
            fontnames.includes(next_font_name)
        ) {
            return DEFAULT_FONT_NAME;
        }
        try {
            await invoke("save_font", {
                oldName: DEFAULT_FONT_NAME,
                newName: next_font_name,
            });
            await get_font_names(null);
            if (fontnames.includes(next_font_name)) {
                return next_font_name;
            }
        } catch (e) {
            console.warn("save generated font with toolset name failed", e);
        }
        return DEFAULT_FONT_NAME;
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
        clear_collision_debug();
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
                checkCollision: collision_check_enabled,
            });
        } catch (e) {
            clear_collision_debug();
            error_msg = format_error_message(e);
            open_alert_dialog("Compile Error", error_msg);
            ready_to_compile = true;
        }
    }

    async function compile_content(event) {
        if (glyph_set == "" || config_name == "" || kerning_name == "") {
            return;
        }
        clear_collision_debug();
        ready_to_compile = false;
        try {
            await kerning_editor_ref.save(event);
            await config_editor_ref.save(event);
            await glyph_data_editor_1.save(event);
            await glyph_data_editor_2.save(event);
            await invoke("run_compile_content", {
                glyphSet: glyph_set,
                configName: config_name,
                kerningName: kerning_name,
                content,
                checkCollision: collision_check_enabled,
            });
        } catch (e) {
            clear_collision_debug();
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
                    <button class="ui-button-secondary" onclick={get_content} title="Load content" aria-label="Load content">
                        <FolderOpen class="h-4 w-4" />
                    </button>
                    <button class="ui-button-secondary" onclick={save_content_same_name} title="Save content" aria-label="Save content">
                        <Save class="h-4 w-4" />
                    </button>
                    <button class="ui-button-ghost" onclick={save_content_as} title="Save content as" aria-label="Save content as">
                        <CopyPlus class="h-4 w-4" />
                    </button>
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
                    <button class="ui-button-secondary" onclick={delete_font} title="Delete font" aria-label="Delete font">
                        <Trash2 class="h-4 w-4" />
                    </button>
                    <button class="ui-button-secondary" onclick={save_font_with_tool_set_name} title="Save font as selected toolset name" aria-label="Save font as selected toolset name">
                        <BookmarkPlus class="h-4 w-4" />
                    </button>
                    <button class="ui-button-ghost" onclick={save_font} title="Save font as" aria-label="Save font as">
                        <CopyPlus class="h-4 w-4" />
                    </button>
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
                    <button class="ui-button-ghost" onclick={save_tool_set_as} title="Save tool set as" aria-label="Save tool set as">
                        <CopyPlus class="h-4 w-4" />
                    </button>
                </div>
            </div>

            <div class="space-y-1">
                <div class="ui-label"></div>
                <div class="flex items-center gap-2">
                <button class="ui-button-secondary" onclick={refresh_all_lists} title="Refresh content, font, toolset, and glyph-set lists" aria-label="Refresh all lists">
                    <RefreshCcw class="h-4 w-4" />
                </button>
                <label class="ui-label flex items-center gap-1 rounded-md border border-[hsl(var(--border))] px-2 py-1">
                    <input type="checkbox" bind:checked={collision_check_enabled} />
                    Collision
                </label>
                <button class="ui-button-secondary" onclick={toggle_theme_mode} title="Toggle theme" aria-label="Toggle theme">
                    {#if theme_mode == "dark"}
                        <Sun class="h-4 w-4" />
                    {:else}
                        <Moon class="h-4 w-4" />
                    {/if}
                </button>
                <button class="ui-button-secondary" onclick={compile_content} disabled={!ready_to_compile} title="Compile only composite glyphs used in current content">
                    Fast
                </button>
                <button class="ui-button-primary flex items-center justify-center" onclick={compile} disabled={!ready_to_compile}>
                    <Hammer class="h-4 w-4" />
                </button>
                <button class="ui-button-secondary" onclick={open_evolution_dialog}>
                    Evolution
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
                    style:font-size={char_size + "px"}
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
                    on_config_error={handle_config_editor_error}
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
        <div class="fixed inset-0 z-[65] flex items-center justify-center bg-black/50 p-4">
            <div class="ui-card w-full max-w-md p-4">
                <div class="text-sm font-semibold">{save_prompt_title}</div>
                <input
                    class="ui-input mt-3 w-full"
                    bind:this={save_prompt_input_ref}
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
                {#if collision_debug_image_url != ""}
                    <div class="mt-3">
                        <div class="text-xs text-[hsl(var(--muted-foreground))]">
                            Collision raster preview
                            {#if collision_debug_payload?.character}
                                ({collision_debug_payload.character})
                            {/if}
                        </div>
                        <img
                            class="mt-2 max-h-[280px] rounded-md border border-[hsl(var(--input))] bg-white p-2"
                            src={collision_debug_image_url}
                            alt="Collision raster preview"
                            style="image-rendering: pixelated;"
                        />
                    </div>
                {/if}
                <div class="mt-3 flex justify-end">
                    <button class="ui-button-primary" onclick={close_alert_dialog}>OK</button>
                </div>
            </div>
        </div>
    {/if}

    <EvolutionDialog
        open={evolution_dialog_open}
        evolution_config_names={evolution_config_names}
        bind:evolution_config_name
        bind:evolution_config_data
        evolution_generation={evolution_generation}
        evolution_base_rendering={evolution_base_rendering}
        evolution_selected_rendering={evolution_selected_rendering}
        evolution_renderings={evolution_renderings}
        bind:evolution_selected_config_data
        bind:evolution_selected_kerning_data
        bind:collision_check_enabled
        evolve_disabled={!ready_to_compile}
        {char_size}
        {get_evolution_config_data}
        {save_evolution_config_same_name}
        {save_evolution_config_as}
        {rename_evolution_config}
        {delete_evolution_config}
        {evolve_shell}
        {evolve_from_rendering}
        {render_again_from_rendering}
        {reset_evolution_renderings}
        {cache_selected_variant_draft}
        {close_evolution_dialog}
        {cache_current_evolution_draft}
        {choose_evolution_rendering}
        {copy_selected_variant_to_main_editors}
        {replace_rendering_from_base}
        {save_selected_variant_as}
    />
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
